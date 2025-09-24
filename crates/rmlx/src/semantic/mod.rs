mod element;
mod enumeration;
mod expression;
mod group;
mod loader;
mod model;
mod structure;
mod symbol;
mod unresolved_schema;

use crate::semantic::symbol::{LazySymbol, SymbolKind};
use crate::{
    semantic::{
        loader::load_rmlx,
        model::SchemaModel,
        symbol::{Symbol, SymbolRef},
        unresolved_schema::UnresolvedSchema,
    },
    RmlxParser,
};
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

pub struct Workspace {
    unresolved: HashMap<String, UnresolvedSchema>,
    model: Arc<RwLock<SchemaModel>>,
    current_namespace: Option<String>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            unresolved: HashMap::default(),
            model: Arc::new(RwLock::new(SchemaModel::default())),
            current_namespace: None,
        }
    }
}

impl Workspace {
    fn load_single_model(&mut self, path: &Url) {
        let content = load_rmlx(path).unwrap();
        let ast = RmlxParser::build_ast(&content);
        if self.unresolved.contains_key(&content) {
            return;
        }

        let mut unresolved_module = UnresolvedSchema::new(&ast, path.path(), self);
        let namespace = unresolved_module.namespace().map(str::to_string);
        self.current_namespace.clone_from(&namespace);

        {
            let mut model = self.model.write().unwrap();
            if let Some(ns) = &namespace {
                model.namespaces.entry(ns.clone()).or_default();
            }
        }

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
        self.current_namespace.clone_from(&namespace);
    }

    fn get_target_or_local_namespace<'s>(&'s self, namespace: Option<&'s str>) -> Option<&'s str> {
        if let Some(ns) = namespace {
            Some(ns)
        } else {
            self.current_namespace.as_deref()
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

pub trait TypeResolver<T> {
    fn resolve(&mut self, workspace: &mut Workspace) -> bool;
    fn as_resolved_type(&self) -> T;
}

#[cfg(test)]
mod tests {
    use crate::{semantic::symbol::SymbolKind, Workspace};
    use url::Url;

    #[test]
    fn test() {
        let path = "/home/irisu/Storage/Projects/rml/examples/schema.rmlx";
        let url = Url::from_file_path(path).unwrap();

        let mut workspace = Workspace::default();
        workspace.load_single_model(&url);
        println!();
    }
}
