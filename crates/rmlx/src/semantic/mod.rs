mod element;
mod enumeration;
mod expression;
mod group;
mod loader;
mod model;
mod structure;
mod symbol;
mod unresolved_schema;

pub use model::SchemaModel;

use crate::semantic::symbol::{LazySymbol, SymbolKind};
use crate::{
    RmlxParser,
    semantic::{
        loader::load_rmlx,
        symbol::{Symbol, SymbolRef},
        unresolved_schema::UnresolvedSchema,
    },
};
use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::Deref;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use url::Url;

pub struct UnresolvedType {
    generic_base: Option<String>,
    namespace: Option<String>,
    identifier: String,
}

pub struct AnalysisWorkspace {
    source: String,
    path: Url,
    model: Arc<RwLock<SchemaModel>>,

    namespace_stack: Vec<Option<String>>,
    unresolved: HashMap<String, UnresolvedSchema>,
}

impl Debug for AnalysisWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisWorkspace")
            .field("path", &self.path)
            .field("namespace_stack", &self.namespace_stack)
            .finish()
    }
}

impl AnalysisWorkspace {
    #[must_use]
    pub fn new(path: Url) -> Self {
        Self {
            source: String::new(),
            path,
            unresolved: HashMap::default(),
            model: Arc::new(RwLock::new(SchemaModel::default())),
            namespace_stack: vec![],
        }
    }

    #[must_use]
    pub fn model(&self) -> Arc<RwLock<SchemaModel>> {
        self.model.clone()
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn run(mut self) -> Self {
        self.source = load_rmlx(&self.path).unwrap();
        let source = self.source.clone();
        let path = self.path.clone();
        self.load_model_internal(&source, &path);
        self
    }

    fn load_model_internal(&mut self, source: &str, path: &Url) {
        if self.unresolved.contains_key(path.as_str()) {
            return;
        }

        let mut unresolved_module = UnresolvedSchema::new(source, path.to_file_path().unwrap().to_str().unwrap(), self);
        let namespace = unresolved_module.namespace().map(str::to_string);

        {
            let mut model = self.model.write().unwrap();
            if let Some(ns) = &namespace {
                model.namespaces.entry(ns.clone()).or_default();
            }
        }

        self.namespace_stack.push(namespace);
        loop {
            let mut symbols = unresolved_module.resolve(self);
            if symbols.is_empty() {
                if unresolved_module.is_empty() {
                    break;
                }
                self.unresolved.insert(path.to_string(), unresolved_module);
                break;
            }

            symbols.retain(|f| {
                let mut model = self.model.write().unwrap();
                if let Some(r) = f.try_get_self_reference(&model) {
                    model.replace_type(r, f.clone());
                    return false;
                }

                true
            });

            let mut write = self.model.write().unwrap();
            if let Some(ns) = unresolved_module.namespace() {
                let vec = write.namespaces.get_mut(ns).unwrap();
                vec.extend(symbols);
            } else {
                write.global.extend(symbols);
            }
        }
        self.namespace_stack.pop();
    }

    pub(crate) fn load_single_model(&mut self, path: &Url) {
        let content = load_rmlx(path).unwrap();
        self.load_model_internal(&content, path);
    }

    fn get_target_or_local_namespace<'s>(&'s self, namespace: Option<&'s str>) -> Option<&'s str> {
        if let Some(ns) = namespace {
            Some(ns)
        } else if let Some(namespace) = self.namespace_stack.last() {
            namespace.as_deref()
        } else {
            None
        }
    }

    fn get_type(&mut self, ty: &UnresolvedType) -> Option<SymbolRef> {
        let mut model = self.model.write().unwrap();
        let namespace = self.get_target_or_local_namespace(ty.namespace.as_deref());
        let identifier = if let Some(generic) = &ty.generic_base {
            format!("{generic}_{}", ty.identifier)
        } else {
            ty.identifier.clone()
        };

        if let Some(id) = model.get_type_id(namespace, &identifier) {
            return Some(SymbolRef {
                id,
                namespace: namespace.map(str::to_string),
            });
        } else if let Some(generic) = &ty.generic_base
            && let Some(generic) = model.get_type_by_name(ty.namespace.as_deref(), generic).unwrap()
            && let Some((target_ref, target)) = model.get_type_by_name(None, &ty.identifier).unwrap_with_ref()
        {
            let generic = generic.as_generic_symbol();
            let ct = generic.construct_type(target, &target_ref);
            model.add_symbol(ty.namespace.as_deref(), ct);
        }
        //Base type does not found

        None
    }

    fn create_self_reference(&mut self, ty: &UnresolvedType) -> SymbolRef {
        let namespace = self.get_target_or_local_namespace(ty.namespace.as_deref());
        let mut model = self.model.write().unwrap();
        let type_table = model.get_mut_type_table(namespace);
        let id = type_table.len();
        type_table.push(SymbolKind::Lazy(LazySymbol {
            source: id,
            identifier: ty.identifier.clone(),
        }));
        SymbolRef {
            id,
            namespace: namespace.map(str::to_string),
        }
    }
}

pub trait TypeResolver<T> {
    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> bool;
    fn as_resolved_type(&self) -> T;
}

pub struct AnalyzerState {
    group: SymbolRef,
    allowed: Vec<usize>,
}

pub struct PreviousElement {
    name: String,
    namespace: Option<String>,
    state: usize,
}

pub struct RmlAnalyzer {
    model: Arc<RwLock<SchemaModel>>,
    depth: Vec<PreviousElement>,
    states: Vec<AnalyzerState>,
    active: usize,
}

impl RmlAnalyzer {
    fn build_states(root: &SymbolRef, model: &SchemaModel) -> Vec<AnalyzerState> {
        let mut states = Vec::new();
        let mut visited = HashSet::new();
        let mut to_process = vec![root.clone()];

        // Build a mapping from SymbolRef to state index
        let mut state_indices = HashMap::new();

        while let Some(current_symbol) = to_process.pop() {
            if visited.contains(&current_symbol) {
                continue;
            }
            visited.insert(current_symbol.clone());

            if let Some(ty) = model.get_type_by_id(current_symbol.namespace.as_deref(), current_symbol.id) {
                let group = ty.as_group_symbol();

                // Get all reachable groups from this state
                let reachable_groups: Vec<SymbolRef> = group.groups().iter().map(|g| g.symbol().clone()).collect();

                // Add new symbols to processing queue
                for symbol in &reachable_groups {
                    if !visited.contains(symbol) && !to_process.contains(symbol) {
                        to_process.push(symbol.clone());
                    }
                }
                // Create state with placeholder allowed indices (will be filled later)
                let state = AnalyzerState {
                    group: current_symbol.clone(),
                    allowed: Vec::new(), // will populate after all states are created
                };

                state_indices.insert(current_symbol.clone(), states.len());
                states.push(state);
            }
        }

        // Now populate the allowed transitions
        for state in &mut states {
            let current_symbol = &state.group;
            if let Some(ty) = model.get_type_by_id(current_symbol.namespace.as_deref(), current_symbol.id) {
                let group = ty.as_group_symbol();
                let reachable_groups: Vec<SymbolRef> = group.groups().iter().map(|g| g.symbol().clone()).collect();

                state.allowed = reachable_groups
                    .iter()
                    .filter_map(|symbol| state_indices.get(symbol))
                    .copied()
                    .collect();
            }
        }

        states
    }

    #[must_use]
    pub fn new(model: Arc<RwLock<SchemaModel>>) -> Self {
        let read_model = model.read().unwrap();
        let (id, namespace) = read_model.get_root_group_ref();
        let group = SymbolRef { namespace, id };

        let states = Self::build_states(&group, &read_model);
        drop(read_model);

        Self {
            model,
            depth: vec![],
            states,
            active: 0,
        }
    }

    #[must_use]
    pub fn is_allowed_element(&self, namespace: Option<&str>, name: &str) -> bool {
        let model = self.model.read().unwrap();
        let element = model.get_type_by_name(namespace, name).as_element_symbol().unwrap();
        let bind_group = element.group();

        let group_ref = &self.states[self.active].group;
        let ty = model
            .get_type_by_id(group_ref.namespace.as_deref(), group_ref.id)
            .unwrap();
        let group = ty.as_group_symbol();
        let groups = group.groups();
        groups.iter().any(|g| {
            let symbol = g.symbol();
            symbol.id == bind_group.id && symbol.namespace == bind_group.namespace
        })
    }

    fn next_state(&mut self, namespace: Option<&str>, name: &str) {
        debug_assert!(self.is_allowed_element(namespace, name));

        let model = self.model.read().unwrap();
        let element = model.get_type_by_name(namespace, name).as_element_symbol().unwrap();
        let bind_group = element.group();

        let ty = model
            .get_type_by_id(bind_group.namespace.as_deref(), bind_group.id)
            .unwrap();
        let group = ty.as_group_symbol();
        if group.groups().is_empty() {
            self.depth.push(PreviousElement {
                name: name.to_string(),
                namespace: namespace.map(str::to_string),
                state: self.active,
            });
            return;
        }

        let active = &self.states[self.active];
        let next = *active
            .allowed
            .iter()
            .find(|allowed| {
                let state = &self.states[**allowed];
                state.group.id == bind_group.id && state.group.namespace == bind_group.namespace
            })
            .unwrap();

        self.depth.push(PreviousElement {
            name: name.to_string(),
            namespace: namespace.map(str::to_string),
            state: self.active,
        });
        self.active = next;
    }

    fn exit_state(&mut self, namespace: Option<&str>, name: &str) {
        let previous_element = self.depth.pop().unwrap();
        assert!(previous_element.name == name && previous_element.namespace.as_deref() == namespace);
        self.active = previous_element.state;
    }

    fn is_valid_attribute(&self, name: &str, value: &str) -> bool {
        let model = self.model.read().unwrap();
        let last_element = self.depth.last().unwrap();
        let element = model
            .get_type_by_name(last_element.namespace.as_deref(), &last_element.name)
            .as_element_symbol()
            .unwrap();
        let field = element.field(name).unwrap();
        let field_type = model.get_type_by_ref(field.ty());
        let field_type = field_type.as_ref();
        field_type.can_parse(value, &model)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnalysisWorkspace, RmlAnalyzer};
    use url::Url;

    #[test]
    fn test() {
        let path = "D:\\Projects\\rml\\examples\\schema.rmlx";
        let url = Url::from_file_path(path).unwrap();
        let mut workspace = AnalysisWorkspace::new(url).run();
        let mut rml = RmlAnalyzer::new(workspace.model.clone());
        let _allowed = rml.is_allowed_element(None, "Node");
        rml.next_state(None, "Node");
        let _allowed_attribute = rml.is_valid_attribute("left", "10px");
        let _allowed_generic = rml.is_valid_attribute("aspect_ratio", "10");
        println!();
    }
}
