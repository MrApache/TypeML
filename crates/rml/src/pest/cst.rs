use crate::pest::Rule;
use lexer_core::CstKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RmlNode {
    File,
    Ident,
    String,
    Number,
    NumberWithUnit,
    Boolean,

    EnumValue,
    Struct,
    StructField,
    FieldName,
    FieldValue,
    DirectiveContent,
    Directive,
    Element,
    EmptyTag,
    Tag,
    AttrValue,
    Attribute,
    Expression,
    ExprName,
    ExprArg,
    ArgName,
    ArgValue,
    ListValue,
    Negation,

    Symbol,
}

impl CstKind for RmlNode {
    type Rule = Rule;

    fn map_rule_to_cst_kind(rule: Self::Rule) -> Self {
        match rule {
            Rule::file => RmlNode::File,
            Rule::identifier => RmlNode::Ident,
            Rule::number => RmlNode::Number,
            Rule::boolean => RmlNode::Boolean,
            Rule::string => RmlNode::String,
            Rule::enum_val => RmlNode::EnumValue,
            Rule::number_with_unit => RmlNode::NumberWithUnit,
            Rule::structure => RmlNode::Struct,
            Rule::struct_field => RmlNode::StructField,
            Rule::field_name => RmlNode::FieldName,
            Rule::field_value => RmlNode::FieldValue,
            Rule::directive_content => RmlNode::DirectiveContent,
            Rule::directive => RmlNode::Directive,
            Rule::element => RmlNode::Element,
            Rule::empty_tag => RmlNode::EmptyTag,
            Rule::tag => RmlNode::Tag,
            Rule::attr_value => RmlNode::AttrValue,
            Rule::attribute => RmlNode::Attribute,
            Rule::expression => RmlNode::Expression,
            Rule::expr_name => RmlNode::ExprName,
            Rule::expr_arg => RmlNode::ExprArg,
            Rule::arg_name => RmlNode::ArgName,
            Rule::arg_val => RmlNode::ArgValue,
            Rule::list_val => RmlNode::ListValue,
            Rule::negation => RmlNode::Negation,
            _ => RmlNode::Symbol,
        }
    }
}
