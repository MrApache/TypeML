use crate::cst::RmlNode;
use crate::unresolved::implements::{Impl, ImplKind};
use lexer_core::CstNode;

#[derive(Debug)]
pub enum FieldValue {
    String(String),
    Number(String),
    Enum(String),
    Boolean(bool),
}

impl FieldValue {
    fn build(node: &CstNode<RmlNode>) -> FieldValue {
        let child = node.children.first().unwrap();
        match child.kind {
            RmlNode::String => FieldValue::String(node.text.to_string()),
            RmlNode::Number => FieldValue::Number(node.text.to_string()),
            RmlNode::Boolean => FieldValue::Boolean(str::parse(&node.text).unwrap()),
            RmlNode::EnumValue => FieldValue::Enum(node.text.to_string()),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct StructField {
    pub identifier: String,
    pub value: FieldValue,
}

impl StructField {
    fn build(node: &CstNode<RmlNode>) -> StructField {
        let mut iter = node.children.iter();
        let identifier = iter.next().unwrap().text.clone();
        let value = FieldValue::build(iter.next().unwrap());
        StructField { identifier, value }
    }
}

#[derive(Debug)]
pub struct Struct {
    pub source: String,
    pub fields: Vec<StructField>,
}

impl Struct {
    pub fn build_struct_fields(node: &CstNode<RmlNode>) -> Vec<StructField> {
        node.children
            .iter()
            .filter(|c| matches!(c.kind, RmlNode::StructField))
            .map(StructField::build)
            .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub enum StructKind {
    Ref(String),
    Impl(Struct),
}

impl StructKind {
    pub fn as_str(&self) -> &str {
        match self {
            StructKind::Ref(r) => r.as_str(),
            StructKind::Impl(i) => i.source.as_str(),
        }
    }

    pub fn as_struct<'a>(&'a self, impls: &'a [Impl]) -> &'a Struct {
        match self {
            StructKind::Ref(r) => impls
                .iter()
                .find(|i| i.identifier.as_str() == r)
                .map(|i| match &i.kind {
                    ImplKind::Struct(s) => s,
                    ImplKind::Expr(_) => unreachable!(),
                })
                .unwrap(),
            StructKind::Impl(i) => i,
        }
    }

    pub fn build(node: &CstNode<RmlNode>) -> Self {
        let source = node
            .text
            .strip_prefix("{{")
            .unwrap()
            .strip_suffix("}}")
            .unwrap()
            .to_string();

        let child = node.children.first().unwrap();
        match child.kind {
            RmlNode::ImplRef => StructKind::Ref(child.children.first().unwrap().text.clone()),
            RmlNode::StructFields => StructKind::Impl(Struct {
                source,
                fields: Struct::build_struct_fields(child),
            }),
            _ => unreachable!(),
        }
    }
}
