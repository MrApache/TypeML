use crate::Rule;
use lexer_core::CstKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RmlxNode {
    Ident,
    Number,
    String,
    Boolean,
    BaseType,

    Keyword,
    Symbol,
    Comment,
    Whitespace,

    Hash,
    LT,
    GT,

    Array,
    Plus,
    Star,
    QMark,
    Arrow,
    AT,

    Directive,
    DirectiveContent,

    Annotation,
    AnnotationValue,

    Attribute,
    AttributeList,

    Enum,
    EnumVariant,

    SimpleField,
    SimpleFields,
    Block,

    GroupContent,
    GroupEntry,
    Group,

    File,
    Struct,
    Element,
    Expression,
    GenericType,
    NsIdent,
    Count,
    CustomType,
}

impl CstKind for RmlxNode {
    type Rule = Rule;

    fn map_rule_to_cst_kind(rule: Self::Rule) -> Self {
        match rule {
            Rule::ident => RmlxNode::Ident,
            Rule::number => RmlxNode::Number,
            Rule::string => RmlxNode::String,
            Rule::boolean => RmlxNode::Boolean,
            Rule::base_types => RmlxNode::BaseType,
            Rule::directive => RmlxNode::Directive,

            Rule::UNIQUE | Rule::STRUCT | Rule::ENUM | Rule::ELEMENT | Rule::EXPRESSION | Rule::GROUP => {
                RmlxNode::Keyword
            }

            Rule::COMMENT_MULTI | Rule::COMMENT_LINE => RmlxNode::Comment,
            Rule::WHITESPACE => RmlxNode::Whitespace,

            Rule::file => RmlxNode::File,
            Rule::r#struct => RmlxNode::Struct,
            Rule::r#enum => RmlxNode::Enum,
            Rule::element => RmlxNode::Element,
            Rule::block => RmlxNode::Block,
            Rule::simple_field => RmlxNode::SimpleField,
            Rule::simple_fields => RmlxNode::SimpleFields,
            Rule::annotation => RmlxNode::Annotation,
            Rule::attribute => RmlxNode::Attribute,
            Rule::attribute_list => RmlxNode::AttributeList,
            Rule::generic_type => RmlxNode::GenericType,
            Rule::ns_ident => RmlxNode::NsIdent,
            Rule::group_content => RmlxNode::GroupContent,
            Rule::group => RmlxNode::Group,
            Rule::expression => RmlxNode::Expression,
            Rule::count => RmlxNode::Count,
            Rule::group_entry => RmlxNode::GroupEntry,
            Rule::custom_types => RmlxNode::CustomType,

            Rule::array => RmlxNode::Array,
            Rule::LT => RmlxNode::LT,
            Rule::GT => RmlxNode::GT,
            Rule::HASH => RmlxNode::Hash,
            Rule::PLUS => RmlxNode::Plus,
            Rule::STAR => RmlxNode::Star,
            Rule::QMARK => RmlxNode::QMark,
            Rule::ARROW => RmlxNode::Arrow,
            Rule::AT => RmlxNode::AT,
            Rule::directive_content => RmlxNode::DirectiveContent,
            Rule::annotation_value => RmlxNode::AnnotationValue,
            Rule::enum_variant => RmlxNode::EnumVariant,
            _ => RmlxNode::Symbol,
        }
    }
}
