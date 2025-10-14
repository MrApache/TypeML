use crate::semantic::element::ElementSymbol;
use crate::semantic::group::GroupSymbol;
use crate::semantic::symbol::{
    F32, F64, GenericSymbol, I8, I16, I32, I64, Str, Symbol, SymbolKind, SymbolRef, U8, U16, U32, U64,
};
use pest::pratt_parser::Op;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SchemaModel {
    pub namespaces: Vec<String>,
    pub modules: Vec<Vec<SymbolKind>>,
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
            namespaces: vec![String::new()], //The empty string is the global namespace.
            modules: vec![global],
        }
    }
}

impl SchemaModel {
    #[must_use]
    pub fn get_type_by_ref(&self, symbol_ref: SymbolRef) -> TypeQuery<'_> {
        let type_table = self.get_type_table_by_namespace_id(symbol_ref.namespace);
        let ty = type_table.get(symbol_ref.id).unwrap();
        TypeQuery {
            symbol_ref,
            kind: Some(ty),
        }
    }

    #[must_use]
    pub fn get_type_by_name(&self, namespace: usize, name: &str) -> TypeQuery<'_> {
        let type_table = self.get_type_table_by_namespace_id(namespace);
        let ty = type_table
            .iter()
            .enumerate()
            .find(|(id, kind)| kind.identifier() == name);

        if let Some((id, kind)) = ty {
            TypeQuery {
                symbol_ref: SymbolRef { namespace, id },
                kind: Some(kind),
            }
        } else {
            TypeQuery {
                symbol_ref: SymbolRef::default(),
                kind: None,
            }
        }
    }

    #[must_use]
    pub fn get_namespace_id(&self, namespace: Option<&str>) -> usize {
        self.try_get_namespace_id(namespace)
            .unwrap_or_else(|| panic!("Namespace '{}' does not exist", namespace.unwrap()))
    }

    //TODO Temp method
    #[must_use]
    pub fn try_get_namespace_id(&self, namespace: Option<&str>) -> Option<usize> {
        if let Some(ns) = namespace {
            self.namespaces
                .iter()
                .enumerate()
                .find(|(id, n)| *n == ns)
                .map(|(id, _)| id)
        } else {
            Some(0)
        }
    }

    #[must_use]
    pub fn get_type_table_by_namespace_name(&self, namespace: Option<&str>) -> &[SymbolKind] {
        let id = self.get_namespace_id(namespace);
        self.get_type_table_by_namespace_id(id)
    }

    #[must_use]
    pub fn get_type_table_by_namespace_id(&self, namespace: usize) -> &[SymbolKind] {
        self.modules.get(namespace).unwrap()
    }

    pub fn get_mut_type_table_by_namespace_name(&mut self, namespace: Option<&str>) -> &mut Vec<SymbolKind> {
        let id = self.get_namespace_id(namespace);
        self.get_mut_type_table_by_namespace_id(id)
    }

    pub fn get_mut_type_table_by_namespace_id(&mut self, namespace: usize) -> &mut Vec<SymbolKind> {
        self.modules.get_mut(namespace).unwrap()
    }

    #[must_use]
    pub fn get_type_id(&self, namespace: usize, name: &str) -> Option<usize> {
        let type_table = self.get_type_table_by_namespace_id(namespace);
        type_table.iter().position(|t| t.identifier() == name)
    }

    #[must_use]
    pub fn get_type_by_id(&self, namespace: Option<&str>, id: usize) -> Option<&SymbolKind> {
        let type_table = self.get_type_table_by_namespace_name(namespace);
        type_table.get(id)
    }

    pub fn add_symbol(&mut self, namespace: usize, symbol: SymbolKind) {
        let type_table = self.get_mut_type_table_by_namespace_id(namespace);
        type_table.push(symbol);
    }

    pub fn replace_type(&mut self, symbol_ref: &SymbolRef, kind: SymbolKind) {
        let type_table = self.get_mut_type_table_by_namespace_id(symbol_ref.namespace);
        type_table[symbol_ref.id] = kind;
    }

    #[must_use]
    pub fn get_root_group_ref(&self) -> SymbolRef {
        let (namespace, id) = self
            .modules
            .iter()
            .enumerate()
            .find_map(|(namespace, array)| {
                array
                    .iter()
                    .position(|k| k.identifier() == "Root")
                    .map(|id| (namespace, id))
            })
            .unwrap();

        SymbolRef { namespace, id }
    }

    #[must_use]
    pub fn get_main_group_ref(&self) -> SymbolRef {
        let id = self
            .modules
            .iter()
            .find_map(|array| array.iter().position(|k| k.identifier() == "Main"))
            .unwrap();

        SymbolRef { namespace: 0, id }
    }
}

pub struct TypeQuery<'a> {
    symbol_ref: SymbolRef,
    kind: Option<&'a SymbolKind>,
}

impl<'a> TypeQuery<'a> {
    pub fn is_element_symbol(&self) -> bool {
        self.kind.is_some_and(SymbolKind::is_element_symbol)
    }

    pub fn as_element_symbol(&self) -> Option<&'a ElementSymbol> {
        self.kind.and_then(|k| {
            if k.is_element_symbol() {
                Some(k.as_element_symbol())
            } else {
                None
            }
        })
    }

    pub fn is_group_symbol(&self) -> bool {
        self.kind.is_some_and(SymbolKind::is_group_symbol)
    }

    pub fn as_group_symbol(&self) -> Option<&'a GroupSymbol> {
        self.kind.and_then(|k| {
            if k.is_group_symbol() {
                Some(k.as_group_symbol())
            } else {
                None
            }
        })
    }

    pub fn as_ref(&self) -> &SymbolKind {
        self.kind.unwrap()
    }

    pub fn unwrap(self) -> Option<&'a SymbolKind> {
        self.kind
    }

    pub fn unwrap_with_ref(self) -> Option<(SymbolRef, &'a SymbolKind)> {
        if let Some(value) = self.kind {
            Some((self.symbol_ref, value))
        } else {
            None
        }
    }
}
