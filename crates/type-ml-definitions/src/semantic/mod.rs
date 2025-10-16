mod element;
mod enumeration;
mod expression;
mod group;
mod loader;
mod model;
mod structure;
mod symbol;
mod unresolved_schema;

pub use expression::{ExpressionField, ExpressionSymbol};
pub use group::{GroupConfig, GroupSymbol};
pub(crate) use loader::LoadError;
pub use model::SchemaModel;
pub use symbol::{Symbol, SymbolRef};

use crate::semantic::symbol::{LazySymbol, SymbolKind};
use crate::semantic::{loader::load_tmd, unresolved_schema::UnresolvedSchema};
use std::collections::HashMap;
use std::fmt::Debug;
use url::Url;

#[derive(Debug)]
pub struct UnresolvedType {
    generic_base: Option<String>,
    namespace: Option<String>,
    identifier: String,
}

#[derive(Debug)]
pub struct AnalysisWorkspace {
    source: String,
    path: Url,
    model: SchemaModel,

    namespace_stack: Vec<usize>,
    unresolved: HashMap<String, UnresolvedSchema>,
}

impl AnalysisWorkspace {
    #[must_use]
    pub fn new(path: Url) -> Self {
        Self {
            source: String::new(),
            path,
            unresolved: HashMap::default(),
            model: SchemaModel::default(),
            namespace_stack: vec![],
        }
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn run(mut self) -> Result<SchemaModel, crate::Error> {
        self.source = load_tmd(&self.path)?;
        let source = self.source.clone();
        let path = self.path.clone();
        self.load_model_internal(&source, &path)?;
        self.model.post_load()?;
        if !self.unresolved.is_empty()
            && let Some((path, schema)) = self.unresolved.into_iter().next()
        {
            return Err(crate::Error::UnresolvedType(
                path,
                schema.next_unresolved().unwrap().to_string(),
            ));
        }
        Ok(self.model)
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
            let mut symbols = unresolved_module.resolve(self)?;
            if symbols.is_empty() {
                if unresolved_module.is_empty() {
                    break;
                }
                self.unresolved.insert(path.to_string(), unresolved_module);
                break;
            }

            symbols.retain(|f| {
                if let Some(r) = f.try_get_self_reference(&self.model) {
                    self.model.replace_type(r, f.clone());
                    return false;
                }

                true
            });

            let vec = self.model.modules.get_mut(namespace_id).expect("Unreachable!");
            vec.extend(symbols);
        }
        self.namespace_stack.pop();

        Ok(())
    }

    pub(crate) fn load_single_model(&mut self, path: &Url) -> Result<(), crate::Error> {
        let content = load_tmd(path)?;
        self.load_model_internal(&content, path)
    }

    fn get_or_add_namespace_id(&mut self, namespace: Option<&str>) -> usize {
        if let Some(ns) = namespace {
            if let Some(id) = self.model.try_get_namespace_id(namespace) {
                return id;
            }

            let id = self.model.namespaces.len();
            self.model.namespaces.push(ns.to_string());
            self.model.modules.push(Vec::new());
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

        let identifier = if let Some(generic) = &ty.generic_base {
            format!("{generic}_{}", ty.identifier)
        } else {
            ty.identifier.clone()
        };

        ns_iter.find_map(|namespace| {
            let namespace = *namespace;
            if let Some(id) = self.model.get_type_id(namespace, &identifier) {
                Some(SymbolRef { namespace, id })
            } else if let Some(generic) = &ty.generic_base
                && let Some(generic) = self.model.get_type_by_name(namespace, generic).unwrap()
                && let Some((target_ref, target)) = self.model.get_type_by_name(0, &ty.identifier).unwrap_with_ref()
            {
                let generic = generic.as_generic_symbol();
                let ct = generic.construct_type(target, target_ref);
                self.model.add_symbol(namespace, ct);
                None
            } else {
                None
            }
        })
    }

    fn create_self_reference(&mut self, ty: &UnresolvedType) -> SymbolRef {
        let namespace = self.get_or_add_namespace_id(ty.namespace.as_deref());
        let type_table = self.model.get_mut_type_table_by_namespace_id(namespace);
        let id = type_table.len();
        type_table.push(SymbolKind::Lazy(LazySymbol {
            source: id,
            identifier: ty.identifier.clone(),
        }));
        SymbolRef { namespace, id }
    }
}

pub trait TypeResolver<T> {
    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> Result<bool, crate::Error>;
    fn as_resolved_type(&self) -> T;
}
