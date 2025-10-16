use crate::cst::RmlNode;
use crate::unresolved::expression::ExpressionKind;
use crate::unresolved::structure::StructKind;
use lexer_core::CstNode;

#[derive(Debug)]
pub enum AttributeValue {
    Boolean(bool),
    Number(String),
    String(String),
    Enum(String),
    Struct(StructKind),
    Expression(ExpressionKind),
}

impl AttributeValue {
    fn build(node: &CstNode<RmlNode>) -> AttributeValue {
        let child = node.children.first().unwrap();
        match child.kind {
            RmlNode::EnumValue => AttributeValue::Enum(node.text.to_string()),
            RmlNode::Number => AttributeValue::Number(node.text.to_string()),
            RmlNode::Boolean => AttributeValue::Boolean(str::parse(&node.text).unwrap()),
            RmlNode::String => AttributeValue::String(node.text.to_string()),
            RmlNode::Expression => AttributeValue::Expression(ExpressionKind::build(child)),
            RmlNode::Struct => AttributeValue::Struct(StructKind::build(child)),
            _ => unreachable!(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AttributeValue::Boolean(value) => {
                if *value {
                    "true"
                } else {
                    "false"
                }
            }
            AttributeValue::Number(value) => value.as_str(),
            AttributeValue::String(value) => value.as_str(),
            AttributeValue::Enum(value) => value.as_str(),
            AttributeValue::Struct(value) => value.as_str(),
            AttributeValue::Expression(_) => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub identifier: String,
    pub value: AttributeValue,
}

impl Attribute {
    pub fn build(node: &CstNode<RmlNode>) -> Self {
        let mut iter = node.children.iter();
        let identifier = iter.next().unwrap().text.clone();
        let value = AttributeValue::build(iter.next().unwrap());
        Attribute { identifier, value }
    }
}
