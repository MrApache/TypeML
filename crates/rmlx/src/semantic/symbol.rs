use crate::Error;
use crate::semantic::expression::ExpressionSymbol;
use crate::semantic::{
    element::ElementSymbol,
    enumeration::{EnumSymbol, EnumVariant},
    group::GroupSymbol,
    model::SchemaModel,
    structure::StructSymbol,
};
use enum_dispatch::enum_dispatch;
use std::{collections::HashMap, fmt::Debug};

#[derive(Default, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SymbolRef {
    pub namespace: usize,
    pub id: usize,
}

#[derive(Debug, Clone)]
pub enum TypeRef {
    Concrete(SymbolRef),
    Generic(String),
}

impl TypeRef {
    pub fn as_concrete(&self) -> SymbolRef {
        match self {
            TypeRef::Concrete(symbol_ref) => *symbol_ref,
            TypeRef::Generic(_) => unreachable!(),
        }
    }
}

#[enum_dispatch]
pub trait Symbol {
    fn identifier(&self) -> &str;
    fn can_parse(&self, value: &str, model: &SchemaModel) -> Result<bool, Error> {
        Ok(false)
    }
    fn try_get_self_reference(&self, model: &SchemaModel) -> Option<&SymbolRef> {
        None
    }
}

macro_rules! impl_symbol {
    ($name:ident, $ident:expr, $parse:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name;
        impl Symbol for $name {
            fn identifier(&self) -> &str {
                $ident
            }

            fn can_parse(&self, value: &str, model: &SchemaModel) -> Result<bool, crate::Error> {
                Ok($parse(value).is_ok())
            }
        }
    };
}

impl_symbol!(F32, "f32", str::parse::<f32>);
impl_symbol!(F64, "f64", str::parse::<f64>);
impl_symbol!(I8, "i8", str::parse::<i8>);
impl_symbol!(I16, "i16", str::parse::<i16>);
impl_symbol!(I32, "i32", str::parse::<i32>);
impl_symbol!(I64, "i64", str::parse::<i64>);
impl_symbol!(U8, "u8", str::parse::<u8>);
impl_symbol!(U16, "u16", str::parse::<u16>);
impl_symbol!(U32, "u32", str::parse::<u32>);
impl_symbol!(U64, "u64", str::parse::<u64>);
impl_symbol!(Str, "String", str::parse::<String>);

#[derive(Debug, Clone)]
pub struct ArraySymbol {
    identifier: String,
    inner: SymbolRef,
}

impl Default for ArraySymbol {
    fn default() -> Self {
        Self {
            identifier: "Array".to_string(),
            inner: SymbolRef::default(),
        }
    }
}

impl Symbol for ArraySymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn can_parse(&self, value: &str, model: &SchemaModel) -> Result<bool, Error> {
        let mut result = true;
        value.split(',').map(str::trim).try_for_each(|value| {
            let kind = model.get_type_by_ref(self.inner).unwrap().unwrap();
            result &= kind.can_parse(value, model)?;
            Ok::<_, Error>(())
        })?;
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct GenericSymbol {
    base: SymbolKind,
}

impl GenericSymbol {
    #[must_use]
    pub const fn new(base: SymbolKind) -> Self {
        Self { base }
    }

    #[must_use]
    pub fn option() -> Self {
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

    pub fn array() -> Self {
        Self {
            base: SymbolKind::Array(ArraySymbol::default()),
        }
    }

    #[must_use]
    pub fn construct_type(&self, other: &SymbolKind, other_ref: SymbolRef) -> SymbolKind {
        match &self.base {
            SymbolKind::Array(value) => SymbolKind::Array(ArraySymbol {
                identifier: format!("Array_{}", other.identifier()),
                inner: other_ref,
            }),
            SymbolKind::Struct(value) => SymbolKind::Struct(StructSymbol {
                identifier: format!("{}_{}", value.identifier(), other.identifier()),
                fields: value.fields.clone(),
                metadata: value.metadata.clone(),
            }),
            SymbolKind::Enum(value) => {
                let variants = value
                    .variants
                    .iter()
                    .map(|var| {
                        let ty = var.ty.as_ref().map(|ty| match ty {
                            TypeRef::Concrete(concrete) => TypeRef::Concrete(*concrete),
                            TypeRef::Generic(generic) => TypeRef::Concrete(other_ref),
                        });
                        EnumVariant {
                            identifier: var.identifier.clone(),
                            ty,
                            pattern: var.pattern.clone(),
                        }
                    })
                    .collect::<Vec<_>>();
                SymbolKind::Enum(EnumSymbol {
                    identifier: format!("{}_{}", value.identifier(), other.identifier()),
                    variants,
                    metadata: value.metadata.clone(),
                })
            }
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
    Expression(ExpressionSymbol),
    Array(ArraySymbol),
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

    pub fn as_element_symbol(&self) -> &ElementSymbol {
        match self {
            SymbolKind::Element(symbol) => symbol,
            _ => panic!("Not a element symbol"),
        }
    }

    pub fn is_group_symbol(&self) -> bool {
        matches!(self, SymbolKind::Group(_))
    }

    pub fn is_element_symbol(&self) -> bool {
        matches!(self, SymbolKind::Element(_))
    }
}
