use crate::pest::Rule;
use lexer_core::CstKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RmlNode {
    File,

    Ident,
    NsIdent,
    Alias,

    String,
    Boolean,

    Integer,
    Float,
    Number,

    EnumValue,
    NumberWithUnit,

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

            Rule::ident => RmlNode::Ident,
            Rule::ns_ident => RmlNode::NsIdent,
            Rule::alias => RmlNode::Alias,
            Rule::string => RmlNode::String,
            Rule::boolean => RmlNode::Boolean,

            Rule::number => RmlNode::Number,
            //Rule::integer => RmlNode::Integer,
            //Rule::float => RmlNode::Float,
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
