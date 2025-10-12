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
    semantic::{
        loader::load_rmlx,
        symbol::{Symbol, SymbolRef},
        unresolved_schema::UnresolvedSchema,
    },
    RmlxParser,
};
use std::fmt::Debug;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use std::ops::Deref;
use url::Url;
use crate::semantic::group::GroupConfig;

#[derive(Debug, Default, Clone)]
pub enum Namespace {
    #[default]
    Global,
    Custom(String),
    Current,
}

impl Namespace {
    //fn as_deref(&self) -> Option<&str> {
    //    match self {
    //        Namespace::Global => None,
    //        Namespace::Custom(custom) => Some(custom.as_str()),
    //        Namespace::Current => None,
    //    }
    //}
}

pub struct UnresolvedType {
    generic_base: Option<String>,
    namespace: Namespace,
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
                if let Some(r) = f.try_get_self_reference() {
                    let mut model = self.model.write().unwrap();
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

    fn get_target_or_local_namespace(&self, namespace: &Namespace) -> Option<&str> {
        if let Some(ns) = namespace {
            Some(ns)
        } else if let Some(namespace) = self.namespace_stack.last() {
            namespace.as_deref()
        }
        else {
            None
        }
    }

    fn get_type(&mut self, ty: &UnresolvedType) -> Option<SymbolRef> {
        let mut model = self.model.write().unwrap();
        let namespace = self.get_target_or_local_namespace(ty.namespace);
        let identifier = if let Some(generic) = &ty.generic_base {
            format!("{generic}_{}", ty.identifier)
        } else {
            ty.identifier.clone()
        };

        if let Some(id) = model.get_type_id(namespace, &identifier) {
            return Some(SymbolRef {
                id,
                model: self.model.clone(),
                namespace: ty.namespace.clone(),
            });
        } else if let Some(generic) = &ty.generic_base
            && let Some(generic) = model.get_type_by_name(ty.namespace.as_deref(), generic)
            && let Some(target) = model.get_type_by_name(None, &ty.identifier)
        {
            let generic = generic.as_generic_symbol();
            let ct = generic.construct_type(target);
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
            model: self.model.clone(),
            namespace: namespace.map(str::to_string),
        }
    }
}

pub struct RmlAnalyzer {
    model: Arc<RwLock<SchemaModel>>,
    group: SymbolRef,
}

impl RmlAnalyzer {
    #[must_use]
    pub fn new(model: Arc<RwLock<SchemaModel>>) -> Self {
        let read_model = model.read().unwrap();
        let (id, namespace) = read_model.get_root_group_ref();
        let group = SymbolRef {
            namespace,
            id,
            model: model.clone(),
        };

        drop(read_model);

        Self {
            model,
            group
        }
    }

    #[must_use]
    pub fn is_allowed_element(&self, namespace: Option<&str>, name: &str) -> bool {
        let model = self.model.read().unwrap();
        let type_table = model.get_type_table(namespace);
        let element = type_table.iter()
            .filter(|kind| kind.is_element_symbol())
            .find(|kind| {
                let element = kind.as_element_symbol();
                element.identifier() == name
            })
            .unwrap();
        let element = element.as_element_symbol();
        let bind_group = element.group();

        let ty = model.get_type_by_id(self.group.namespace.as_deref(), self.group.id).unwrap();
        let group = ty.as_group_symbol();
        let groups = group.groups();
        groups.iter().any(|g| {
            let symbol = g.symbol();
            symbol.id == bind_group.id && symbol.namespace == bind_group.namespace
        })
    }
}

pub trait TypeResolver<T> {
    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> bool;
    fn as_resolved_type(&self) -> T;
}

#[cfg(test)]
mod tests {
    use crate::{semantic::symbol::SymbolKind, AnalysisWorkspace, RmlAnalyzer};
    use url::Url;

    #[test]
    fn test() {
        let path = "D:\\Projects\\rml\\examples\\schema.rmlx";
        let url = Url::from_file_path(path).unwrap();
        let mut workspace = AnalysisWorkspace::new(url).run();
        let rml = RmlAnalyzer::new(workspace.model.clone());
        let _allowed = rml.is_allowed_element(None, "Node");
        println!();
    }
}
