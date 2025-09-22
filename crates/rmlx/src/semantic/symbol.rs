use crate::semantic::{element::ElementSymbol, enumeration::EnumSymbol, group::GroupSymbol, model::SchemaModel, structure::StructSymbol};
use enum_dispatch::enum_dispatch;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SymbolRef {
    pub id: usize,
    pub model: Arc<RwLock<SchemaModel>>,
}

#[enum_dispatch]
pub trait Symbol {
    fn identifier(&self) -> &str;
}

macro_rules! impl_symbol {
    ($name:ident, $ident:expr) => {
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

pub struct GenericSymbol {
    base: SymbolKind,
}

impl GenericSymbol {
    pub fn new(base: SymbolKind) -> GenericSymbol {
        Self {
            base,
        }
    }
}

impl Symbol for GenericSymbol {
    fn identifier(&self) ->  &str {
        &self.base.identifier()
    }
}

impl Symbol for Box<GenericSymbol> {
    fn identifier(&self) -> &str {
        &self.base.identifier()
    }
}

#[enum_dispatch(Symbol)]
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
    Element(ElementSymbol)
}

impl SymbolKind {
    pub fn as_generic_symbol(&self) -> &GenericSymbol {
        match self {
            SymbolKind::Generic(generic_symbol) => &generic_symbol,
            _ => unreachable!()
        }
    }
}
