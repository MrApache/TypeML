use crate::{
    semantic::symbol::{
        GenericSymbol, Str, Symbol, SymbolKind, SymbolRef, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8,
    },
    CstNode,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct SchemaModel {
    pub namespaces: HashMap<String, Vec<SymbolKind>>,
    pub global: Vec<SymbolKind>,
}

impl Default for SchemaModel {
    fn default() -> Self {
        let global = vec![
            SymbolKind::F32(F32),
            SymbolKind::F64(F64),
            SymbolKind::I8(I8),
            SymbolKind::I16(I16),
            SymbolKind::I32(I32),
            SymbolKind::I64(I64),
            SymbolKind::U8(U8),
            SymbolKind::U16(U16),
            SymbolKind::U32(U32),
            SymbolKind::U64(U64),
            SymbolKind::String(Str),
            SymbolKind::Generic(Box::new(GenericSymbol::option())),
        ];
        Self {
            namespaces: HashMap::default(),
            global,
        }
    }
}

impl SchemaModel {
    #[must_use]
    pub fn get_type_table(&self, namespace: Option<&str>) -> &[SymbolKind] {
        if let Some(ns) = namespace {
            self.namespaces
                .get(ns)
                .unwrap_or_else(|| panic!("Namespace '{ns}' does not exist"))
        } else {
            &self.global
        }
    }

    pub fn get_mut_type_table(&mut self, namespace: Option<&str>) -> &mut Vec<SymbolKind> {
        if let Some(ns) = namespace {
            self.namespaces
                .get_mut(ns)
                .unwrap_or_else(|| panic!("Namespace '{ns}' does not exist"))
        } else {
            &mut self.global
        }
    }

    #[must_use]
    pub fn get_type_id(&self, namespace: Option<&str>, name: &str) -> Option<usize> {
        let type_table = self.get_type_table(namespace);
        if let Some(id) = type_table.iter().position(|t| t.identifier() == name) {
            Some(id)
        } else if namespace.is_some() {
            self.get_type_id(None, name)
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_type_by_id(&self, namespace: Option<&str>, id: usize) -> Option<&SymbolKind> {
        let type_table = self.get_type_table(namespace);
        type_table.get(id)
    }

    #[must_use]
    pub fn get_type_by_name(&self, namespace: Option<&str>, name: &str) -> Option<&SymbolKind> {
        let type_table = self.get_type_table(namespace);
        type_table.iter().find(|t| t.identifier() == name)
    }

    pub fn add_symbol(&mut self, namespace: Option<&str>, symbol: SymbolKind) {
        let type_table = self.get_mut_type_table(namespace);
        type_table.push(symbol);
    }

    pub fn replace_type(&mut self, symbol_ref: &SymbolRef, kind: SymbolKind) {
        let type_table = self.get_mut_type_table(symbol_ref.namespace.as_deref());
        type_table[symbol_ref.id] = kind;
    }

    #[must_use]
    pub fn get_root_group_ref(&self) -> (usize, Option<String>) {
        let root = self.namespaces.iter()  // (&String, &Vec<Kind>)
            .flat_map(|(key, kinds)| {
                kinds.iter()
                    .enumerate()
                    .map(move |(idx, kind)| (idx, Some(key.clone()), kind))
            })
            .chain(
                self.global.iter()
                    .enumerate()
                    .map(|(idx, kind)| (idx, None, kind))
            )
            .find(|(_, _, t)| t.identifier() == "Root")
            .map(|(index, namespace, _)| (index, namespace))
            .unwrap();

        root
    }
}
