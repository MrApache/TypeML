use crate::semantic::{
    element::ElementSymbol,
    enumeration::{EnumSymbol, EnumVariant},
    group::GroupSymbol,
    model::SchemaModel,
    structure::StructSymbol,
};
use enum_dispatch::enum_dispatch;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct SymbolRef {
    pub namespace: Option<String>,
    pub id: usize,
    pub model: Arc<RwLock<SchemaModel>>,
}

impl Debug for SymbolRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolRef")
            .field("namespace", &self.namespace)
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum TypeRef {
    Concrete(SymbolRef),
    Generic(String),
}

#[enum_dispatch]
pub trait Symbol {
    fn identifier(&self) -> &str;
    fn try_get_self_reference(&self) -> Option<&SymbolRef> {
        None
    }
}

macro_rules! impl_symbol {
    ($name:ident, $ident:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name;
        impl Symbol for $name {
            fn identifier(&self) -> &str {
                $ident
            }
        }
    };
}

impl_symbol!(F32, "f32");
impl_symbol!(F64, "f64");
impl_symbol!(I8, "i8");
impl_symbol!(I16, "i16");
impl_symbol!(I32, "i32");
impl_symbol!(I64, "i64");
impl_symbol!(U8, "u8");
impl_symbol!(U16, "u16");
impl_symbol!(U32, "u32");
impl_symbol!(U64, "u64");
impl_symbol!(Str, "String");

#[derive(Debug, Clone)]
pub struct GenericSymbol {
    base: SymbolKind,
}

impl GenericSymbol {
    #[must_use]
    pub const fn new(base: SymbolKind) -> GenericSymbol {
        Self { base }
    }

    #[must_use]
    pub fn option() -> GenericSymbol {
        let variants = vec![
            EnumVariant {
                identifier: "Some".to_string(),
                ty: Some(TypeRef::Generic("T".to_string())),
                pattern: None,
            },
            EnumVariant {
                identifier: "None".to_string(),
                ty: None,
                pattern: None,
            },
        ];
        GenericSymbol {
            base: SymbolKind::Enum(EnumSymbol {
                identifier: "Option".to_string(),
                variants,
                metadata: HashMap::default(),
            }),
        }
    }

    #[must_use]
    pub fn construct_type(&self, other: &SymbolKind) -> SymbolKind {
        match &self.base {
            SymbolKind::Struct(value) => SymbolKind::Struct(StructSymbol {
                identifier: format!("{}_{}", value.identifier(), other.identifier()),
                fields: value.fields.clone(),
                metadata: value.metadata.clone(),
            }),
            SymbolKind::Enum(value) => SymbolKind::Enum(EnumSymbol {
                identifier: format!("{}_{}", value.identifier(), other.identifier()),
                variants: value.variants.clone(),
                metadata: value.metadata.clone(),
            }),
            SymbolKind::Generic(_) => todo!("Make type construction"),
            SymbolKind::Group(_) => unimplemented!("The group does not support generics"),
            SymbolKind::Element(_) => unimplemented!("The element does not support generics"),
            _ => unreachable!("The default types does not support generics"), // Default types
        }
    }
}

impl Symbol for GenericSymbol {
    fn identifier(&self) -> &str {
        self.base.identifier()
    }
}

impl Symbol for Box<GenericSymbol> {
    fn identifier(&self) -> &str {
        self.base.identifier()
    }
}

#[derive(Debug, Clone)]
pub struct LazySymbol {
    pub source: usize,
    pub identifier: String,
} 

impl Symbol for LazySymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[enum_dispatch(Symbol)]
#[derive(Debug, Clone)]
pub enum SymbolKind {
    F32(F32),
    F64(F64),
    I8(I8),
    I16(I16),
    I32(I32),
    I64(I64),
    U8(U8),
    U16(U16),
    U32(U32),
    U64(U64),
    String(Str),
    Generic(Box<GenericSymbol>),
    Struct(StructSymbol),
    Enum(EnumSymbol),
    Group(GroupSymbol),
    Element(ElementSymbol),
    Lazy(LazySymbol),
}

impl SymbolKind {
    pub fn as_generic_symbol(&self) -> &GenericSymbol {
        match self {
            SymbolKind::Generic(symbol) => symbol,
            _ => panic!("Not a generic symbol"),
        }
    }

    pub fn as_group_symbol(&self) -> &GroupSymbol {
        match self {
            SymbolKind::Group(symbol) => symbol,
            _ => panic!("Not a group symbol"),
        }
    }

    pub fn is_group_symbol(&self) -> bool {
        matches!(self, SymbolKind::Group(_))
    }

    pub fn is_element_symbol(&self) -> bool {
        matches!(self, SymbolKind::Element(_))
    }

    pub fn as_element_symbol(&self) -> &ElementSymbol {
        match self {
            SymbolKind::Element(symbol) => symbol,
            _ => panic!("Not a element symbol"),
        }
    }
}
