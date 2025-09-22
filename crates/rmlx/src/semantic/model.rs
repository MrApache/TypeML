use crate::semantic::{enumeration::{EnumSymbol, EnumVariant}, symbol::{Str, Symbol, SymbolKind, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8}};
use std::collections::HashMap;

pub struct SchemaModel {
    pub namespaces: HashMap<String, Vec<SymbolKind>>,
    pub global: Vec<SymbolKind>,
}

impl Default for SchemaModel {
    fn default() -> Self {
        let mut global = vec![];
        global.push(SymbolKind::F32(F32));
        global.push(SymbolKind::F64(F64));
        global.push(SymbolKind::I8(I8));
        global.push(SymbolKind::I16(I16));
        global.push(SymbolKind::I32(I32));
        global.push(SymbolKind::I64(I64));
        global.push(SymbolKind::U8(U8));
        global.push(SymbolKind::U16(U16));
        global.push(SymbolKind::U32(U32));
        global.push(SymbolKind::U64(U64));
        global.push(SymbolKind::String(Str));
        //global.push(SymbolKind::Enum(EnumSymbol {
        //    identifier: "Option".to_string(),
        //    variants: vec![
        //        EnumVariant {
        //            identifier: ,
        //            ty: todo!(),
        //            pattern: todo!()
        //        }
        //    ],
        //    metadata: HashMap::default(),
        //}));
        Self {
            namespaces: HashMap::default(),
            global,
        }
    }
}

impl SchemaModel {
    pub fn empty() -> Self {
        Self {
            namespaces: HashMap::default(),
            global: vec![],
        }
    }

    pub fn get_type_id(&self, namespace: Option<&str>, name: &str) -> Option<usize> {
        let type_table = if let Some(ns) = namespace {
            self.namespaces
                .get(ns)
                .expect(&format!("Namespace '{ns}' does not exist"))
        } else {
            &self.global
        };

        type_table.iter().position(|t| t.identifier() == name)
    }
}
