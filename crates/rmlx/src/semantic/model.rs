use crate::semantic::element::ElementSymbol;
use crate::semantic::expression::ExpressionSymbol;
use crate::semantic::group::GroupSymbol;
use crate::semantic::symbol::{
    F32, F64, GenericSymbol, I8, I16, I32, I64, Str, Symbol, SymbolKind, SymbolRef, U8, U16, U32, U64,
};
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
            SymbolKind::Generic(Box::new(GenericSymbol::array())),
        ];

        Self {
            namespaces: vec![String::new()], //The empty string is the global namespace.
            modules: vec![global],
        }
    }
}

impl SchemaModel {
    fn find_duplicate_identifiers(&mut self) -> Result<(), crate::Error> {
        self.modules
            .iter()
            .zip(self.namespaces.iter())
            .try_for_each(|(module, namespace)| {
                let mut occurrences: HashMap<&str, usize> = HashMap::new();
                for kind in module {
                    *occurrences.entry(kind.identifier()).or_default() += 1;
                }

                for (identifier, indices) in &occurrences {
                    if *indices > 1 {
                        return Err(crate::Error::AlreadyDefinedType(
                            namespace.clone(),
                            (*identifier).to_string(),
                        ));
                    }
                }

                Ok::<_, crate::Error>(())
            })?;

        Ok(())
    }

    pub(crate) fn post_load(&mut self) -> Result<(), crate::Error> {
        self.find_duplicate_identifiers()?;

        let root_ref = self.get_root_group_ref()?;
        let global = self.modules.first_mut().unwrap();

        //Create main group
        let main_group = GroupSymbol::main(root_ref);
        global.push(SymbolKind::Group(main_group));

        //TODO Remove generic Array type
        /*
        let array_index = global
            .iter()
            .enumerate()
            .find(|(_, s)| s.identifier() == "Array")
            .map(|(i, _)| i)
            .unwrap();
        global.remove(array_index);
        */

        Ok(())
    }
    #[must_use]
    pub fn get_type_by_ref(&self, symbol_ref: SymbolRef) -> TypeQuery<'_> {
        let type_table = self.get_type_table_by_namespace_id(symbol_ref.namespace);
        let ty = type_table.get(symbol_ref.id).expect("Unreachable!");
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
            .find(|(_, kind)| kind.identifier() == name);

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

    pub fn get_namespace_id(&self, namespace: Option<&str>) -> Result<usize, crate::Error> {
        if let Some(ns) = namespace {
            self.namespaces
                .iter()
                .enumerate()
                .find(|(_, n)| *n == ns)
                .map(|(id, _)| id)
                .ok_or(crate::Error::NamespaceNotFound(ns.to_string()))
        } else {
            Ok(0)
        }
    }

    #[must_use]
    pub fn try_get_namespace_id(&self, namespace: Option<&str>) -> Option<usize> {
        if let Some(ns) = namespace {
            self.namespaces
                .iter()
                .enumerate()
                .find(|(_, n)| *n == ns)
                .map(|(id, _)| id)
        } else {
            Some(0)
        }
    }

    pub fn get_type_table_by_namespace_name(&self, namespace: Option<&str>) -> Result<&[SymbolKind], crate::Error> {
        let id = self.get_namespace_id(namespace)?;
        Ok(self.get_type_table_by_namespace_id(id))
    }

    #[must_use]
    pub fn get_type_table_by_namespace_id(&self, namespace: usize) -> &[SymbolKind] {
        self.modules.get(namespace).expect("Unreachable!")
    }

    pub fn get_mut_type_table_by_namespace_name(
        &mut self,
        namespace: Option<&str>,
    ) -> Result<&mut Vec<SymbolKind>, crate::Error> {
        let id = self.get_namespace_id(namespace)?;
        Ok(self.get_mut_type_table_by_namespace_id(id))
    }

    pub fn get_mut_type_table_by_namespace_id(&mut self, namespace: usize) -> &mut Vec<SymbolKind> {
        self.modules.get_mut(namespace).expect("Unreachable!")
    }

    #[must_use]
    pub fn get_type_id(&self, namespace: usize, name: &str) -> Option<usize> {
        let type_table = self.get_type_table_by_namespace_id(namespace);
        type_table.iter().position(|t| t.identifier() == name)
    }

    pub fn get_type_by_id(&self, namespace: Option<&str>, id: usize) -> Result<Option<&SymbolKind>, crate::Error> {
        let type_table = self.get_type_table_by_namespace_name(namespace)?;
        Ok(type_table.get(id))
    }

    pub fn add_symbol(&mut self, namespace: usize, symbol: SymbolKind) {
        let type_table = self.get_mut_type_table_by_namespace_id(namespace);
        type_table.push(symbol);
    }

    pub fn replace_type(&mut self, symbol_ref: &SymbolRef, kind: SymbolKind) {
        let type_table = self.get_mut_type_table_by_namespace_id(symbol_ref.namespace);
        type_table[symbol_ref.id] = kind;
    }

    pub fn get_root_group_ref(&self) -> Result<SymbolRef, crate::Error> {
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
            .ok_or(crate::Error::RootGroupNotFound)?;

        Ok(SymbolRef { namespace, id })
    }

    #[must_use]
    pub fn get_main_group_ref(&self) -> SymbolRef {
        let id = self
            .modules
            .iter()
            .find_map(|array| array.iter().position(|k| k.identifier() == "Main"))
            .expect("Unreachable!");

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

    pub fn is_expression_symbol(&self) -> bool {
        self.kind.is_some_and(SymbolKind::is_expression_symbol)
    }

    pub fn as_expression_symbol(&self) -> Option<&'a ExpressionSymbol> {
        self.kind.and_then(|k| {
            if k.is_expression_symbol() {
                Some(k.as_expression_symbol())
            } else {
                None
            }
        })
    }

    pub fn as_ref(&self) -> &SymbolKind {
        self.kind.expect("Unreachable!")
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
