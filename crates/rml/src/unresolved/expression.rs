use crate::cst::RmlNode;
use crate::unresolved::build_ident;
use crate::unresolved::implements::{Impl, ImplKind};
use lexer_core::CstNode;

#[derive(Debug)]
pub struct List {
    pub source: String,
    pub values: Vec<ArgumentValue>,
}

impl List {
    fn build(node: &CstNode<RmlNode>) -> Self {
        let source = node
            .text
            .strip_prefix("[")
            .unwrap()
            .strip_suffix("]")
            .unwrap()
            .to_string();

        let values = node.children.iter().map(ArgumentValue::build).collect();
        List { source, values }
    }
}

#[derive(Debug)]
pub enum ArgumentValue {
    String(String),
    Number(String),
    Boolean(bool),
    Enum(String),
    ListValue(List),
}

impl ArgumentValue {
    pub fn as_str(&self) -> &str {
        match self {
            ArgumentValue::String(value) => value.as_str(),
            ArgumentValue::Number(value) => value.as_str(),
            ArgumentValue::Boolean(value) => {
                if *value {
                    "true"
                } else {
                    "false"
                }
            }
            ArgumentValue::Enum(value) => value.as_str(),
            ArgumentValue::ListValue(value) => value.source.as_str(),
        }
    }

    fn build(node: &CstNode<RmlNode>) -> Self {
        let child = node.children.first().unwrap();
        match child.kind {
            RmlNode::EnumValue => ArgumentValue::Enum(node.text.to_string()),
            RmlNode::Number => ArgumentValue::Number(node.text.to_string()),
            RmlNode::Boolean => ArgumentValue::Boolean(str::parse(&node.text).unwrap()),
            RmlNode::String => ArgumentValue::String(node.text.to_string()),
            RmlNode::ListValue => ArgumentValue::ListValue(List::build(child)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct ExpressionArgument {
    pub identifier: String,
    pub value: ArgumentValue,
}

impl ExpressionArgument {
    pub fn build(node: &CstNode<RmlNode>) -> Self {
        let mut iter = node.children.iter();
        let identifier = iter.next().unwrap().text.clone();
        let value = ArgumentValue::build(iter.next().unwrap());
        ExpressionArgument { identifier, value }
    }
}

#[derive(Debug)]
pub struct Expression {
    pub namespace: Option<String>,
    pub identifier: String,
    pub arguments: Vec<ExpressionArgument>,
}

impl Expression {
    #[must_use]
    pub fn full_path(&self) -> String {
        let ns = self.namespace.clone().unwrap_or_default();
        format!("{ns}::{}", self.identifier)
    }

    pub fn build_expression_arguments(node: &CstNode<RmlNode>) -> Vec<ExpressionArgument> {
        node.children
            .iter()
            .filter(|c| matches!(c.kind, RmlNode::ExprArg))
            .map(ExpressionArgument::build)
            .collect()
    }
}

#[derive(Debug)]
pub enum ExpressionKind {
    Ref(String),
    Impl(Expression),
}

impl ExpressionKind {
    pub fn as_expr<'a>(&'a self, impls: &'a [Impl]) -> &'a Expression {
        match self {
            ExpressionKind::Ref(r) => impls
                .iter()
                .find(|i| i.identifier.as_str() == r)
                .map(|i| match &i.kind {
                    ImplKind::Expr(e) => e,
                    ImplKind::Struct(_) => unreachable!(),
                })
                .unwrap(),
            ExpressionKind::Impl(i) => i,
        }
    }

    pub fn build(node: &CstNode<RmlNode>) -> Self {
        let mut iter = node.children.iter();
        let child = iter.next().unwrap();
        match child.kind {
            RmlNode::ImplRef => ExpressionKind::Ref(child.children.first().unwrap().text.clone()),
            RmlNode::NsIdent => {
                let (namespace, identifier) = build_ident(child);
                let arguments =
                    Expression::build_expression_arguments(iter.find(|c| matches!(c.kind, RmlNode::ExprArgs)).unwrap());
                ExpressionKind::Impl(Expression {
                    namespace,
                    identifier,
                    arguments,
                })
            }
            _ => unreachable!(),
        }
    }
}
