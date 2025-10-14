mod element;
mod enumeration;
mod expression;
mod group;
mod loader;
mod model;
mod structure;
mod symbol;
mod unresolved_schema;

pub(crate) use loader::LoadError;
pub use model::SchemaModel;

use crate::semantic::group::{GroupConfig, GroupSymbol, UnresolvedGroupConfig, UnresolvedGroupSymbol};
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
    pub model: Arc<RwLock<SchemaModel>>,

    namespace_stack: Vec<usize>,
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

    pub fn run(mut self) -> Result<Self, crate::Error> {
        self.source = load_rmlx(&self.path)?;
        let source = self.source.clone();
        let path = self.path.clone();
        self.load_model_internal(&source, &path);
        {
            let root_ref = self.find_root()?;
            let main_group = GroupSymbol::main(root_ref);
            let mut write = self.model.write().expect("Unreachable!");
            write.modules[0].push(SymbolKind::Group(main_group));
        }
        Ok(self)
    }

    fn find_root(&self) -> Result<SymbolRef, crate::Error> {
        let model = self.model.read().expect("Unreachable!");
        model.get_root_group_ref()
    }

    fn load_model_internal(&mut self, source: &str, path: &Url) -> Result<(), crate::Error> {
        if self.unresolved.contains_key(path.as_str()) {
            return Ok(());
        }

        let mut unresolved_module = UnresolvedSchema::new(
            source,
            path.to_file_path()
                .expect("Unreachable!")
                .to_str()
                .expect("Unreachable!"),
            self,
        )?;
        let namespace = unresolved_module.namespace();
        let namespace_id = self.get_or_add_namespace_id(namespace);

        self.namespace_stack.push(namespace_id);
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
                let mut model = self.model.write().expect("Unreachable!");
                if let Some(r) = f.try_get_self_reference(&model) {
                    model.replace_type(r, f.clone());
                    return false;
                }

                true
            });

            let mut write = self.model.write().expect("Unreachable!");
            let vec = write.modules.get_mut(namespace_id).expect("Unreachable!");
            vec.extend(symbols);
        }
        self.namespace_stack.pop();

        Ok(())
    }

    pub(crate) fn load_single_model(&mut self, path: &Url) -> Result<(), crate::Error> {
        let content = load_rmlx(path)?;
        self.load_model_internal(&content, path)
    }

    fn get_or_add_namespace_id(&mut self, namespace: Option<&str>) -> usize {
        let mut model = self.model.write().expect("Unreachable!");

        if let Some(ns) = namespace {
            if let Some(id) = model.try_get_namespace_id(namespace) {
                return id;
            }

            let id = model.namespaces.len();
            model.namespaces.push(ns.to_string());
            model.modules.push(Vec::new());
            return id;
        }

        if let Some(id) = self.namespace_stack.last() {
            return *id;
        }

        0 //Return global namespace
    }

    fn get_type(&mut self, ty: &UnresolvedType) -> Option<SymbolRef> {
        let namespace = self.get_or_add_namespace_id(ty.namespace.as_deref());

        let mut model = self.model.write().expect("model is not poisoned");
        let identifier = if let Some(generic) = &ty.generic_base {
            format!("{generic}_{}", ty.identifier)
        } else {
            ty.identifier.clone()
        };

        if let Some(id) = model.get_type_id(namespace, &identifier) {
            return Some(SymbolRef { namespace, id });
        } else if let Some(generic) = &ty.generic_base
            && let Some(generic) = model.get_type_by_name(namespace, generic).unwrap()
            && let Some((target_ref, target)) = model.get_type_by_name(0, &ty.identifier).unwrap_with_ref()
        {
            let generic = generic.as_generic_symbol();
            let ct = generic.construct_type(target, &target_ref);
            model.add_symbol(namespace, ct);
        }
        //Base type does not found

        None
    }

    fn create_self_reference(&mut self, ty: &UnresolvedType) -> SymbolRef {
        let namespace = self.get_or_add_namespace_id(ty.namespace.as_deref());
        let mut model = self.model.write().expect("model is not poisoned");
        let type_table = model.get_mut_type_table_by_namespace_id(namespace);
        let id = type_table.len();
        type_table.push(SymbolKind::Lazy(LazySymbol {
            source: id,
            identifier: ty.identifier.clone(),
        }));
        SymbolRef { namespace, id }
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
    fn build_states(root: SymbolRef, model: &SchemaModel) -> Vec<AnalyzerState> {
        let mut states = Vec::new();
        let mut visited = HashSet::new();
        let mut to_process = vec![root];

        // Build a mapping from SymbolRef to state index
        let mut state_indices = HashMap::new();

        while let Some(current_symbol) = to_process.pop() {
            if visited.contains(&current_symbol) {
                continue;
            }
            visited.insert(current_symbol);

            if let Some(ty) = model.get_type_by_ref(current_symbol).unwrap() {
                let group = ty.as_group_symbol();

                // Get all reachable groups from this state
                let reachable_groups: Vec<SymbolRef> = group.groups().iter().map(GroupConfig::symbol).collect();

                // Add new symbols to processing queue
                for symbol in reachable_groups {
                    if !visited.contains(&symbol) && !to_process.contains(&symbol) {
                        to_process.push(symbol);
                    }
                }
                // Create state with placeholder allowed indices (will be filled later)
                let state = AnalyzerState {
                    group: current_symbol,
                    allowed: Vec::new(), // will populate after all states are created
                };

                state_indices.insert(current_symbol, states.len());
                states.push(state);
            }
        }

        // Now populate the allowed transitions
        for state in &mut states {
            if let Some(ty) = model.get_type_by_ref(state.group).unwrap() {
                let group = ty.as_group_symbol();
                let reachable_groups: Vec<SymbolRef> = group.groups().iter().map(GroupConfig::symbol).collect();

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
        let read_model = model.read().expect("Unreachable!");
        let group = read_model.get_main_group_ref();
        let states = Self::build_states(group, &read_model);
        drop(read_model);

        Self {
            model,
            depth: vec![],
            states,
            active: 0,
        }
    }

    pub fn is_allowed_element(&self, namespace: Option<&str>, name: &str) -> Result<bool, crate::Error> {
        let model = self.model.read().expect("Unreachable!");
        let namespace_id = model.get_namespace_id(namespace)?;
        let element = model
            .get_type_by_name(namespace_id, name)
            .as_element_symbol()
            .ok_or(crate::Error::ElementNotFound(name.into()))?;
        let bind_group = element.group();

        let group_ref = self.states[self.active].group;
        let ty = model.get_type_by_ref(group_ref).unwrap().expect("Unreachable!");
        let group = ty.as_group_symbol();
        let groups = group.groups();
        Ok(groups.iter().any(|g| g.symbol() == bind_group))
    }

    pub fn next_state(&mut self, namespace: Option<&str>, name: &str) -> Result<(), crate::Error> {
        debug_assert!(self.is_allowed_element(namespace, name)?);

        let model = self.model.read().expect("Unreachable!");
        let namespace_id = model.get_namespace_id(namespace)?;
        let element = model
            .get_type_by_name(namespace_id, name)
            .as_element_symbol()
            .expect("Unreachable!");
        let bind_group = element.group();

        let ty = model.get_type_by_ref(bind_group).unwrap().expect("Unreachable!");
        let group = ty.as_group_symbol();
        if group.groups().is_empty() {
            self.depth.push(PreviousElement {
                name: name.to_string(),
                namespace: namespace.map(str::to_string),
                state: self.active,
            });
            return Ok(());
        }

        let active = &self.states[self.active];
        let next = *active
            .allowed
            .iter()
            .find(|allowed| {
                let state = &self.states[**allowed];
                state.group == bind_group
            })
            .expect("Unreachable!");

        self.depth.push(PreviousElement {
            name: name.to_string(),
            namespace: namespace.map(str::to_string),
            state: self.active,
        });
        self.active = next;

        Ok(())
    }

    pub fn exit_state(&mut self, namespace: Option<&str>, name: &str) {
        let previous_element = self.depth.pop().expect("Unreachable!");
        assert!(previous_element.name == name && previous_element.namespace.as_deref() == namespace);
        self.active = previous_element.state;
    }

    pub fn is_valid_attribute(&self, name: &str, value: &str) -> Result<bool, crate::Error> {
        let model = self.model.read().expect("Unreachable!");
        let last_element = self.depth.last().expect("Unreachable!");
        let element_namespace = model.get_namespace_id(last_element.namespace.as_deref())?;
        let element = model
            .get_type_by_name(element_namespace, &last_element.name)
            .as_element_symbol()
            .expect("Unreachable!");
        let field = element.field(name).expect("Unreachable!");
        let field_type = model.get_type_by_ref(field.ty());
        let field_type = field_type.as_ref();
        field_type.can_parse(value, &model)
    }
}
