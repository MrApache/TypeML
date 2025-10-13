use crate::semantic::element::ElementSymbol;
use crate::semantic::group::GroupSymbol;
use crate::semantic::symbol::{
    F32, F64, GenericSymbol, I8, I16, I32, I64, Str, Symbol, SymbolKind, SymbolRef, U8, U16, U32, U64,
};
use pest::pratt_parser::Op;
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
    pub fn get_type_by_ref(&self, symbol: &SymbolRef) -> TypeQuery {
        let type_table = self.get_type_table(symbol.namespace.as_deref());
        let ty = type_table.get(symbol.id).unwrap();
        TypeQuery {
            symbol_ref: symbol.clone(),
            kind: Some(ty),
        }
    }

    #[must_use]
    pub fn get_type_by_name(&self, namespace: Option<&str>, name: &str) -> TypeQuery {
        let type_table = self.get_type_table(namespace);
        let ty = type_table
            .iter()
            .enumerate()
            .find(|(id, kind)| kind.identifier() == name);

        if let Some((id, kind)) = ty {
            TypeQuery {
                symbol_ref: SymbolRef {
                    namespace: namespace.map(str::to_string),
                    id,
                },
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
        self.namespaces
            .iter()
            .flat_map(|(key, kinds)| {
                kinds
                    .iter()
                    .enumerate()
                    .map(move |(idx, kind)| (idx, Some(key.clone()), kind))
            })
            .chain(self.global.iter().enumerate().map(|(idx, kind)| (idx, None, kind)))
            .find(|(_, _, t)| t.identifier() == "Root")
            .map(|(index, namespace, _)| (index, namespace))
            .unwrap()
    }
}

pub struct TypeQuery<'a> {
    symbol_ref: SymbolRef,
    kind: Option<&'a SymbolKind>,
}

impl<'a> TypeQuery<'a> {
    pub fn is_element_symbol(&self) -> bool {
        self.kind.map_or(false, |k| k.is_element_symbol())
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
        self.kind.map_or(false, |k| k.is_group_symbol())
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
