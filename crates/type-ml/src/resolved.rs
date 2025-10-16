use std::collections::HashMap;

pub enum ResolvedBaseType {
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Boolean(bool),
    String(String),
}

pub struct ResolvedExpressionField {
    identifier: String,
    value: ResolvedType,
}

impl ResolvedExpressionField {
    pub const fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub const fn value(&self) -> &ResolvedType {
        &self.value
    }
}

pub struct ResolvedExpression {
    identifier: String,
    arguments: Vec<ResolvedExpressionField>,
    metadata: HashMap<String, Option<ResolvedBaseType>>,
}

impl ResolvedExpression {
    pub const fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub const fn arguments(&self) -> &[ResolvedExpressionField] {
        self.arguments.as_slice()
    }

    pub const fn metadata(&self) -> &HashMap<String, Option<ResolvedBaseType>> {
        &self.metadata
    }
}

pub struct ResolvedField {
    identifier: String,
    value: ResolvedBaseType,
}

impl ResolvedField {
    pub const fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub const fn value(&self) -> &ResolvedBaseType {
        &self.value
    }
}

pub struct ResolvedStruct {
    fields: Vec<ResolvedField>,
    metadata: HashMap<String, Option<ResolvedBaseType>>,
}

impl ResolvedStruct {
    pub const fn fields(&self) -> &[ResolvedField] {
        self.fields.as_slice()
    }

    pub const fn metadata(&self) -> &HashMap<String, Option<ResolvedBaseType>> {
        &self.metadata
    }
}

pub enum ResolvedType {
    Base(ResolvedBaseType),
    Enum(String),
    Struct(ResolvedStruct),
    Expression(ResolvedExpression),
    List(Vec<ResolvedType>),
}

pub struct ResolvedAttribute {
    identifier: String,
    value: ResolvedType,
}

impl ResolvedAttribute {
    pub const fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub const fn value(&self) -> &ResolvedType {
        &self.value
    }
}

pub struct ResolvedElement {
    identifier: String,
    attributes: Vec<ResolvedAttribute>,
    children: Vec<ResolvedElement>,
    metadata: HashMap<String, Option<ResolvedBaseType>>,
}

impl ResolvedElement {
    pub const fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub const fn attributes(&self) -> &[ResolvedAttribute] {
        self.attributes.as_slice()
    }

    pub const fn children(&self) -> &[ResolvedElement] {
        self.children.as_slice()
    }

    pub const fn metadata(&self) -> &HashMap<String, Option<ResolvedBaseType>> {
        &self.metadata
    }
}
