use std::fmt::Display;

use crate::pest::cst::{CstKind, CstNode};

#[derive(Debug)]
pub struct SchemaAst {
    pub annotations: Vec<Annotation>,
    pub directives: Vec<Directive>,
    pub custom_types: Vec<CustomType>,
}

#[derive(Debug)]
pub struct Directive {
    pub name: String,
    pub value: Option<String>, // содержимое <...>
}

#[derive(Debug)]
pub struct Annotation {
    pub name: String,
    pub value: Option<AnnotationValue>,
}

#[derive(Debug)]
pub enum AnnotationValue {
    String(String),
    Array(Vec<String>),
}

// Rust-like атрибуты #[attr(...)]
#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<BaseType>,
}

#[derive(Debug, Clone)]
pub enum BaseType {
    Number(String),
    Boolean(bool),
    String(String),
    Ident(String),
}

// Все типы, описываемые в DSL
#[derive(Debug)]
pub enum CustomType {
    Struct(Struct),
    Enum(Enum),
    Element(Element),
    Group(Group),
    Expression(Expression),
}

impl CustomType {
    #[must_use]
    pub const fn is_struct(&self) -> bool {
        matches!(self, CustomType::Struct(_))
    }

    #[must_use]
    pub const fn is_enum(&self) -> bool {
        matches!(self, CustomType::Enum(_))
    }

    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self, CustomType::Element(_))
    }

    #[must_use]
    pub const fn is_group(&self) -> bool {
        matches!(self, CustomType::Group(_))
    }

    #[must_use]
    pub const fn is_expression(&self) -> bool {
        matches!(self, CustomType::Expression(_))
    }

    #[must_use]
    pub const fn unwrap_struct(&self) -> &Struct {
        match self {
            CustomType::Struct(value) => value,
            _ => panic!("Not a struct"),
        }
    }

    #[must_use]
    pub const fn unwrap_enum(&self) -> &Enum {
        match self {
            CustomType::Enum(value) => value,
            _ => panic!("Not an enum"),
        }
    }

    #[must_use]
    pub const fn unwrap_element(&self) -> &Element {
        match self {
            CustomType::Element(value) => value,
            _ => panic!("Not an element"),
        }
    }

    #[must_use]
    pub const fn unwrap_group(&self) -> &Group {
        match self {
            CustomType::Group(value) => value,
            _ => panic!("Not a group"),
        }
    }

    #[must_use]
    pub const fn unwrap_expression(&self) -> &Expression {
        match self {
            CustomType::Expression(value) => value,
            _ => panic!("Not an expression"),
        }
    }
}

#[derive(Debug)]
pub struct Struct {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub generic: Option<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Default, Debug, Clone)]
pub struct TypeRef {
    pub namespace: Option<String>,
    pub ident: TypeIdent,
}

impl Display for TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.namespace {
            write!(f, "{path}::{}", self.ident)
        } else {
            write!(f, "{}", self.ident)
        }
    }
}

impl TypeRef {
    #[must_use]
    pub fn new(path: &str) -> Self {
        if let Some(pos) = path.rfind("::") {
            let namespace = Some(path[..pos].to_string());
            let ident = path[pos + 2..].to_string();
            Self {
                namespace,
                ident: TypeIdent::Simple(ident),
            }
        } else {
            Self {
                namespace: None,
                ident: TypeIdent::Simple(path.to_string()),
            }
        }
    }

    #[must_use]
    pub fn new_generic(path: &str, inner: &str) -> Self {
        if let Some(pos) = path.rfind("::") {
            let namespace = Some(path[..pos].to_string());
            let ident = path[pos + 2..].to_string();
            Self {
                namespace,
                ident: TypeIdent::Generic(ident, Box::new(TypeIdent::Simple(inner.to_string()))),
            }
        } else {
            Self {
                namespace: None,
                ident: TypeIdent::Generic(path.to_string(), Box::new(TypeIdent::Simple(inner.to_string()))),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeIdent {
    Simple(String),
    Generic(String, Box<TypeIdent>),
}

impl Default for TypeIdent {
    fn default() -> Self {
        Self::Simple(String::new())
    }
}

impl Display for TypeIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeIdent::Simple(ident) => write!(f, "{ident}"),
            TypeIdent::Generic(ident, inner) => write!(f, "{ident}_{inner}"),
        }
    }
}

#[derive(Debug)]
pub struct Enum {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub value: Option<TypeRef>,
}

#[derive(Debug)]
pub struct Element {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub bind: TypeRef,
    pub fields: Vec<Field>,
}


#[derive(Debug)]
pub struct Group {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub entries: Vec<GroupEntry>,
    pub extend: bool,
    pub count: Option<Count>,
}

#[derive(Debug)]
pub struct GroupEntry {
    pub unique: bool,
    pub name: String,
    pub count: Option<Count>,
}

#[derive(Debug, Copy, Clone)]
pub enum Count {
    Single(u32),
    Range(u32, u32),
    Asterisk,
    Question,
    Plus,
}

#[derive(Debug)]
pub struct Expression {
    pub attributes: Vec<Attribute>,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub fields: Vec<Field>,
}

fn build_directive(node: &CstNode) -> Directive {
    let mut name = String::new();
    let mut value = None;

    for child in &node.children {
        match child.kind {
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::String | CstKind::DirectiveContent => value = Some(child.text.clone()),
            _ => {}
        }
    }

    Directive { name, value }
}

fn build_annotation(node: &CstNode) -> Annotation {
    let mut name = String::new();
    let mut value = None;

    for child in &node.children {
        match child.kind {
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::String => value = Some(AnnotationValue::String(child.text.clone())),
            CstKind::Block => {
                // можно добавить поддержку массива
                let array = child
                    .children
                    .iter()
                    .filter_map(|c| {
                        if let CstKind::Ident = c.kind {
                            Some(c.text.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                value = Some(AnnotationValue::Array(array));
            }
            _ => {}
        }
    }

    Annotation { name, value }
}

fn build_attributes(node: &CstNode) -> Vec<Attribute> {
    let mut attributes = Vec::new();

    let mut iter = node.children.iter();
    consume_token(&mut iter, &CstKind::Hash, Some("#"));
    consume_token(&mut iter, &CstKind::Symbol, Some("["));

    for child in iter {
        if child.kind == CstKind::Attribute {
            attributes.push(build_attribute(child));
        }
    }

    attributes
}

fn build_attribute(node: &CstNode) -> Attribute {
    let mut name = String::new();
    let mut value = None;
    for child in &node.children {
        match child.kind {
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::BaseType => match child.children.first().unwrap().kind {
                CstKind::Number => value = Some(BaseType::Number(child.text.clone())),
                CstKind::Boolean => value = Some(BaseType::Boolean(child.text == "true")),
                CstKind::String => value = Some(BaseType::String(child.text.clone())),
                CstKind::Ident => value = Some(BaseType::Ident(child.text.clone())),
                _ => {}
            },
            _ => {}
        }
    }
    Attribute { name, value }
}

fn build_custom_type(node: &CstNode) -> CustomType {
    assert!(!node.children.is_empty(), "CustomType node has no children");

    let first_child = &node.children[0];
    match first_child.kind {
        CstKind::Struct => CustomType::Struct(build_struct(first_child)),
        CstKind::Enum => CustomType::Enum(build_enum(first_child)),
        CstKind::Element => CustomType::Element(build_element(first_child)),
        CstKind::ExtendGroup => CustomType::Group(build_group(first_child, true)),
        CstKind::Group => CustomType::Group(build_group(first_child, false)),
        CstKind::Expression => CustomType::Expression(build_expression(first_child)),
        _ => panic!("Unexpected child kind in CustomType: {:?}", first_child.kind),
    }
}

fn build_struct(node: &CstNode) -> Struct {
    let mut attributes = Vec::new();
    let mut name = String::new();
    let mut generic = None;
    let mut fields = Vec::new();

    for child in &node.children {
        match child.kind {
            CstKind::AttributeList => attributes.extend(build_attributes(child)),
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::GenericType => {
                let parts: Vec<&str> = child.text.split('<').collect();
                if parts.len() == 2 {
                    generic = Some(parts[1].trim_end_matches('>').to_string());
                }
            }
            CstKind::Block => fields.extend(build_fields(child)),
            _ => {}
        }
    }

    Struct {
        attributes,
        name,
        generic,
        fields,
    }
}

fn build_fields(block_node: &CstNode) -> Vec<Field> {
    let mut fields = Vec::new();

    for child in &block_node.children {
        if child.kind == CstKind::SimpleFields {
            for field_node in &child.children {
                if field_node.kind == CstKind::SimpleField {
                    fields.push(build_field(field_node));
                }
            }
        }
    }

    fields
}

fn build_field(node: &CstNode) -> Field {
    let mut annotations = Vec::new();
    let mut name = String::new();
    let mut ty = TypeRef::default();

    for child in &node.children {
        match child.kind {
            CstKind::Annotation => annotations.push(build_annotation(child)),
            CstKind::Ident if name.is_empty() => name.clone_from(&child.text),
            CstKind::NsIdent => ty = TypeRef::new(child.text.as_str()),
            CstKind::GenericType => {
                let parts: Vec<&str> = child.text.split('<').collect();
                if parts.len() == 2 {
                    ty = TypeRef::new_generic(parts[0], parts[1].trim_end_matches('>'));
                } else {
                    unimplemented!();
                }
            }
            _ => {}
        }
    }

    Field { annotations, name, ty }
}

fn build_enum(node: &CstNode) -> Enum {
    let mut attributes = Vec::new();
    let mut name = String::new();
    let mut variants = Vec::new();

    for child in &node.children {
        match child.kind {
            CstKind::AttributeList => attributes.extend(build_attributes(child)),
            CstKind::Ident if name.is_empty() => name.clone_from(&child.text),
            _ => variants.push(build_enum_variant(child)),
        }
    }

    Enum {
        attributes,
        name,
        variants,
    }
}

fn build_enum_variant(node: &CstNode) -> EnumVariant {
    let mut annotations = Vec::new();
    let mut name = String::new();
    let mut value = None;

    for child in &node.children {
        match child.kind {
            CstKind::Annotation => annotations.push(build_annotation(child)),
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::NsIdent => value = Some(TypeRef::new(child.text.as_str())),
            CstKind::GenericType => {
                let parts: Vec<&str> = child.text.split('<').collect();
                if parts.len() == 2 {
                    value = Some(TypeRef::new_generic(parts[0], parts[1].trim_end_matches('>')));
                } else {
                    unimplemented!();
                }
            }
            _ => {}
        }
    }

    EnumVariant {
        annotations,
        name,
        value,
    }
}

fn build_element(node: &CstNode) -> Element {
    let mut attributes = Vec::new();
    let mut name = String::new();
    let mut bind = TypeRef::default();
    let mut fields = Vec::new();

    for child in &node.children {
        match child.kind {
            CstKind::AttributeList => attributes.extend(build_attributes(child)),
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::NsIdent => bind = TypeRef::new(child.text.as_str()),
            CstKind::GenericType => {
                let parts: Vec<&str> = child.text.split('<').collect();
                if parts.len() == 2 {
                    bind = TypeRef::new_generic(parts[0], parts[1].trim_end_matches('>'));
                } else {
                    unimplemented!();
                }
            }
            CstKind::Block => fields.extend(build_fields(child)),
            _ => {}
        }
    }

    Element {
        attributes,
        name,
        bind,
        fields,
    }
}

fn build_group(node: &CstNode, extend: bool) -> Group {
    let mut attributes = Vec::new();
    let mut name = String::new();
    let mut entries = Vec::new();
    let mut count = None;

    for child in &node.children {
        match child.kind {
            CstKind::AttributeList => attributes.extend(build_attributes(child)),
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::GroupContent => entries.extend(build_group_entries(child)),
            CstKind::Count => count = Some(build_count(child)),
            _ => {}
        }
    }

    Group {
        attributes,
        name,
        entries,
        extend,
        count,
    }
}

fn build_group_entries(node: &CstNode) -> Vec<GroupEntry> {
    let mut entries = Vec::new();

    for child in &node.children {
        if child.kind == CstKind::GroupEntry {
            entries.push(build_group_entry(child));
        }
    }

    entries
}

fn build_group_entry(node: &CstNode) -> GroupEntry {
    let mut unique = false;
    let mut name = String::new();
    let mut count = None;

    let mut iter = node.children.iter();
    consume_token(&mut iter, &CstKind::Plus, Some("+"));

    for child in iter {
        match child.kind {
            CstKind::NsIdent | CstKind::Ident => name.clone_from(&child.text),
            CstKind::Symbol if child.text == "unique" => unique = true,
            CstKind::Count => count = Some(build_count(child)),
            _ => {}
        }
    }

    GroupEntry { unique, name, count }
}

fn build_count(node: &CstNode) -> Count {
    let mut iter = node.children.iter();
    consume_token(&mut iter, &CstKind::Symbol, Some("("));
    match iter.next().unwrap().text.as_str() {
        "*" => Count::Asterisk,
        "?" => Count::Question,
        "+" => Count::Plus,
        _ if node.text.contains('-') => {
            let parts: Vec<u32> = node.text.split('-').filter_map(|p| p.parse().ok()).collect();
            Count::Range(parts[0], parts[1])
        }
        _ => Count::Single(node.text.parse().unwrap_or(1)),
    }
}

fn build_expression(node: &CstNode) -> Expression {
    let mut attributes = Vec::new();
    let mut annotations = Vec::new();
    let mut name = String::new();
    let mut fields = Vec::new();

    for child in &node.children {
        match child.kind {
            CstKind::AttributeList => attributes.extend(build_attributes(child)),
            CstKind::Annotation => annotations.push(build_annotation(child)),
            CstKind::Ident => name.clone_from(&child.text),
            CstKind::Block => fields.extend(child.children.iter().map(build_field)),
            _ => {}
        }
    }

    Expression {
        attributes,
        annotations,
        name,
        fields,
    }
}

fn consume_token<'a, I>(iter: &mut I, expected_kind: &CstKind, expected_text: Option<&str>)
where
    I: Iterator<Item = &'a CstNode>,
{
    if let Some(node) = iter.next() {
        if node.kind != *expected_kind {
            unreachable!();
        }
        if let Some(text) = expected_text
            && node.text != text
        {
            unreachable!()
        }
        return;
    }
    panic!()
}

#[must_use]
pub fn build_schema_ast(cst: &CstNode) -> SchemaAst {
    let mut annotations = Vec::new();
    let mut directives = Vec::new();
    let mut custom_types = Vec::new();

    for child in &cst.children {
        match child.kind {
            CstKind::Directive => directives.push(build_directive(child)),
            CstKind::Annotation => annotations.push(build_annotation(child)),
            CstKind::CustomType => custom_types.push(build_custom_type(child)),
            _ => {}
        }
    }

    SchemaAst {
        annotations,
        directives,
        custom_types,
    }
}
