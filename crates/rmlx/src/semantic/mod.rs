mod element;
mod enumeration;
mod expression;
mod group;
mod loader;
mod model;
mod structure;
mod symbol;
mod unresolved_schema;

pub use group::{GroupConfig, GroupSymbol};
pub(crate) use loader::LoadError;
pub use model::SchemaModel;
pub use symbol::{Symbol, SymbolRef};

use crate::semantic::group::{UnresolvedGroupConfig, UnresolvedGroupSymbol};
use crate::semantic::symbol::{LazySymbol, SymbolKind};
use crate::{
    RmlxParser,
    semantic::{loader::load_rmlx, unresolved_schema::UnresolvedSchema},
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
        self.load_model_internal(&source, &path)?;
        let mut write = self.model.write().expect("Unreachable!");
        write.post_load()?;
        drop(write);
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
        let mut array = [0usize, 0usize];
        let namespace = self.get_or_add_namespace_id(ty.namespace.as_deref());
        let mut ns_iter = if let Some(last) = self.namespace_stack.last()
            && *last == namespace
        {
            array[0] = namespace;
            array[1] = 0;
            array.iter()
        } else {
            array[0] = 0;
            array[1] = namespace;
            let mut iter = array.iter();
            iter.next();
            iter
        };

        let mut model = self.model.write().expect("model is not poisoned");
        let identifier = if let Some(generic) = &ty.generic_base {
            format!("{generic}_{}", ty.identifier)
        } else {
            ty.identifier.clone()
        };

        ns_iter.find_map(|namespace| {
            let namespace = *namespace;
            if let Some(id) = model.get_type_id(namespace, &identifier) {
                Some(SymbolRef { namespace, id })
            } else if let Some(generic) = &ty.generic_base
                && let Some(generic) = model.get_type_by_name(namespace, generic).unwrap()
                && let Some((target_ref, target)) = model.get_type_by_name(0, &ty.identifier).unwrap_with_ref()
            {
                let generic = generic.as_generic_symbol();
                let ct = generic.construct_type(target, target_ref);
                model.add_symbol(namespace, ct);
                None
            } else {
                None
            }
        })
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
