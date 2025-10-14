use crate::RmlNode;
use lexer_core::CstNode;

#[derive(Debug)]
pub struct LayoutAst {
    pub directives: Vec<Directive>,
    pub root: Option<Element>,
}

#[derive(Debug)]
pub struct Directive {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug)]
pub enum FieldValue {
    String(String),
    Number(String),
    Enum(String),
    Boolean(bool),
}

#[derive(Debug)]
pub struct Field {
    pub identifier: String,
    pub value: FieldValue,
}

#[derive(Debug)]
pub struct Struct {
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub enum ArgumentValue {
    String(String),
    Number(String),
    Boolean(bool),
    Enum(String),
    ListValue(Vec<ArgumentValue>),
    Negation(String),
}

#[derive(Debug)]
pub struct ExpressionArgument {
    pub identifier: String,
    pub value: ArgumentValue,
}

#[derive(Debug)]
pub struct Expression {
    pub identifier: String,
    pub arguments: Vec<ExpressionArgument>,
}

#[derive(Debug)]
pub enum AttributeValue {
    Boolean(bool),
    Number(String),
    String(String),
    Enum(String),
    Struct(Struct),
    Expression(Expression),
}

#[derive(Debug)]
pub struct Attribute {
    pub identifier: String,
    pub value: AttributeValue,
}

#[derive(Debug)]
pub struct Element {
    pub namespace: Option<String>,
    pub identifier: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Element>,
}

fn build_directive(node: &CstNode<RmlNode>) -> Directive {
    let mut name = String::new();
    let mut value = None;

    for child in &node.children {
        match child.kind {
            RmlNode::Ident => name.clone_from(&child.text),
            RmlNode::String | RmlNode::DirectiveContent => value = Some(child.text.clone()),
            _ => unreachable!(),
        }
    }

    Directive { name, value }
}

fn build_list_value(node: &CstNode<RmlNode>) -> Vec<ArgumentValue> {
    node.children.iter().map(build_argument_value).collect()
}

fn build_argument_value(node: &CstNode<RmlNode>) -> ArgumentValue {
    let child = node.children.first().unwrap();
    match child.kind {
        RmlNode::EnumValue => ArgumentValue::Enum(node.text.to_string()),
        RmlNode::Number => ArgumentValue::Number(node.text.to_string()),
        RmlNode::Boolean => ArgumentValue::Boolean(str::parse(&node.text).unwrap()),
        RmlNode::String => ArgumentValue::String(node.text.to_string()),
        RmlNode::ListValue => ArgumentValue::ListValue(build_list_value(child)),
        RmlNode::Negation => ArgumentValue::Negation(node.text.to_string()),
        _ => unreachable!(),
    }
}

fn build_expression_argument(node: &CstNode<RmlNode>) -> ExpressionArgument {
    let mut iter = node.children.iter();
    let identifier = iter.next().unwrap().text.clone();
    let value = build_argument_value(iter.next().unwrap());
    ExpressionArgument { identifier, value }
}

fn build_expression(node: &CstNode<RmlNode>) -> Expression {
    let mut iter = node.children.iter();
    let identifier = iter.next().unwrap().text.clone();
    let arguments = iter
        .filter(|c| matches!(c.kind, RmlNode::ExprArg))
        .map(build_expression_argument)
        .collect::<Vec<_>>();

    Expression { identifier, arguments }
}

fn build_field_value(node: &CstNode<RmlNode>) -> FieldValue {
    let child = node.children.first().unwrap();
    match child.kind {
        RmlNode::String => FieldValue::String(node.text.to_string()),
        RmlNode::Number => FieldValue::Number(node.text.to_string()),
        RmlNode::Boolean => FieldValue::Boolean(str::parse(&node.text).unwrap()),
        RmlNode::EnumValue => FieldValue::Enum(node.text.to_string()),
        _ => unreachable!(),
    }
}

fn build_struct_field(node: &CstNode<RmlNode>) -> Field {
    let mut iter = node.children.iter();
    let identifier = iter.next().unwrap().text.clone();
    let value = build_field_value(iter.next().unwrap());
    Field { identifier, value }
}

fn build_struct(node: &CstNode<RmlNode>) -> Struct {
    let fields = node
        .children
        .iter()
        .filter(|c| matches!(c.kind, RmlNode::StructField))
        .map(build_struct_field)
        .collect::<Vec<_>>();
    Struct { fields }
}

fn build_attribute_value(node: &CstNode<RmlNode>) -> AttributeValue {
    let child = node.children.first().unwrap();
    match child.kind {
        RmlNode::EnumValue => AttributeValue::Enum(node.text.to_string()),
        RmlNode::Number => AttributeValue::Number(node.text.to_string()),
        RmlNode::Boolean => AttributeValue::Boolean(str::parse(&node.text).unwrap()),
        RmlNode::String => AttributeValue::String(node.text.to_string()),
        RmlNode::Expression => AttributeValue::Expression(build_expression(child)),
        RmlNode::Struct => AttributeValue::Struct(build_struct(child)),
        _ => unreachable!(),
    }
}

fn build_attribute(node: &CstNode<RmlNode>) -> Attribute {
    let mut iter = node.children.iter();
    let identifier = iter.next().unwrap().text.clone();
    let value = build_attribute_value(iter.next().unwrap());
    Attribute { identifier, value }
}

fn build_alias(node: &CstNode<RmlNode>) -> String {
    node.children.first().unwrap().text.clone()
}

fn build_element_ident(node: &CstNode<RmlNode>) -> (Option<String>, String) {
    match node.kind {
        RmlNode::Ident => (None, node.text.to_string()),
        RmlNode::NsIdent => {
            if let Some((ns, ident)) = node.text.rsplit_once("::") {
                (Some(ns.to_string()), ident.to_string())
            } else {
                (None, node.text.to_string())
            }
        }
        _ => unreachable!(),
    }
}

fn build_element_from_tag(node: &CstNode<RmlNode>) -> Element {
    let (open_ns, open_ident) = build_element_ident(node.children.first().unwrap());
    let (close_ns, close_ident) = build_element_ident(node.children.last().unwrap());
    let mut alias = String::new();
    let mut children = vec![];
    let mut attributes = vec![];

    node.children[1..node.children.len() - 1]
        .iter()
        .for_each(|c| match c.kind {
            RmlNode::Alias => alias = build_alias(c),
            RmlNode::Element => children.push(build_element(c)),
            RmlNode::Attribute => attributes.push(build_attribute(c)),
            kind => unreachable!("{kind:#?}"),
        });

    //assert_eq!(open_ns, close_ns);
    //assert_eq!(open_ident, close_ident);

    Element {
        namespace: open_ns,
        identifier: open_ident,
        attributes,
        children,
    }
}

fn build_element_from_empty_tag(node: &CstNode<RmlNode>) -> Element {
    let (namespace, identifier) = build_element_ident(node.children.first().unwrap());
    let mut attributes = Vec::new();

    for child in node.children.iter().skip(1) {
        match child.kind {
            RmlNode::Attribute => attributes.push(build_attribute(child)),
            _ => unreachable!(),
        }
    }

    Element {
        namespace,
        identifier,
        attributes,
        children: vec![],
    }
}

fn build_element(node: &CstNode<RmlNode>) -> Element {
    let child = node.children.first().unwrap();
    match child.kind {
        RmlNode::Tag => build_element_from_tag(child),
        RmlNode::EmptyTag => build_element_from_empty_tag(child),
        _ => unreachable!(),
    }
}

#[must_use]
pub fn build_layout_ast(cst: &CstNode<RmlNode>) -> LayoutAst {
    let mut directives = Vec::new();
    let mut root = None;

    for child in &cst.children {
        match child.kind {
            RmlNode::Directive => directives.push(build_directive(child)),
            RmlNode::Element => root = Some(build_element(child)),
            RmlNode::Symbol => {}
            kind => unreachable!("{kind:#?}"),
        }
    }

    LayoutAst { directives, root }
}
