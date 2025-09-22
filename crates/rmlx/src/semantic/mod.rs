mod element;
mod enumeration;
mod group;
mod loader;
mod model;
mod structure;
mod symbol;
mod unresolved_model;
mod expression;

use crate::{
    semantic::{loader::load_rmlx, model::SchemaModel, symbol::{SymbolKind, SymbolRef}, unresolved_model::UnresolvedSchemaModel},
    SchemaAst,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use url::Url;

pub struct UnresolvedType {
    namespace: Option<String>,
    identifier: String,
}

pub struct Workspace {
    unresolved: HashMap<String, UnresolvedSchemaModel>,
    model: Arc<RwLock<SchemaModel>>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            unresolved: HashMap::default(),
            model: Arc::new(RwLock::new(SchemaModel::default())),
        }
    }
}

impl Workspace {
    fn load_single_model(&mut self, path: Url) {
        let content = load_rmlx(&path).unwrap();
        let ast = SchemaAst::new(&content);
        if self.unresolved.contains_key(&content) {
            return;
        }

        let mut model = UnresolvedSchemaModel::new(ast, path.path(), self);
        loop {
            let symbols = model.resolve(self);
            if symbols.is_empty() {
                if model.is_empty() {
                    break;
                }
                self.unresolved.insert(path.to_string(), model);
                break;
            }

            let mut write = self.model.write().unwrap();
            if let Some(ns) = model.namespace() {
                write.namespaces.insert(ns.to_string(), symbols);
            } else {
                write.global.extend(symbols);
            }
        }
    }

    fn get_type(&self, namespace: Option<&str>, name: &str) -> Option<SymbolRef> {
        let model = self.model.read().unwrap();
        if let Some(id) = model.get_type_id(namespace, name) {
            return Some(SymbolRef {
                id,
                model: self.model.clone(),
            });
        }

        None
    }

    fn get_generic_type(&mut self, base: &str, namespace: Option<&str>, name: &str) -> Option<SymbolRef> {
        let mut model = self.model.write().unwrap();
        let generic_name = format!("{base}_{name}");
        if let Some(id) = model.get_type_id(None, &generic_name) {
            return Some(SymbolRef {
                id,
                model: self.model.clone(),
            });
        } else {
            //TODO
            if let Some(id) =  model.get_type_id(None, base) {
                let ty = model.global.get(id).unwrap();
                let generic = ty.as_generic_symbol();
            } 
            //model.global.push(SymbolKind::Generic());
        }

        None
    }
}

pub trait TypeResolver<T> {
    fn resolve(&mut self, workspace: &Workspace) -> bool;
    fn as_resolved_type(&self) -> T;
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::Workspace;

    #[test]
    fn test() {
        let path = "/home/irisu/Storage/Projects/rml/examples/schema.rmlx";
        //let content = std::fs::read_to_string(path).unwrap();
        let mut workspace = Workspace::default();
        let url = Url::from_file_path(path).unwrap();
        workspace.load_single_model(url);
        println!();
    }
}
