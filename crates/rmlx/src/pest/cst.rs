use crate::pest::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct CstNode {
    pub kind: CstKind,
    pub text: String,
    pub children: Vec<CstNode>,
    pub start: usize,       // абсолютная позиция в файле
    pub end: usize,         // абсолютная позиция в файле
    pub delta_line: usize,  // строки от предыдущего токена
    pub delta_start: usize, // смещение в строке
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CstKind {
    Ident,
    Number,
    String,
    Boolean,
    BaseType,

    Keyword,
    Symbol,
    Comment,
    Whitespace,

    File,
    Directive,
    Attribute,
    AttributeList,
    Annotation,
    Struct,
    Enum,
    Element,
    GroupContent,
    Group,
    ExtendGroup,
    Expression,
    Block,
    SimpleField,
    SimpleFields,
    GenericType,
    NsIdent,
    Count,
    GroupEntry,
    CustomType,
}

pub fn build_cst(pair: &Pair<Rule>, prev_line: &mut usize, prev_col: &mut usize) -> CstNode {
    let span = pair.as_span();
    let (start_line, start_col) = span.start_pos().line_col();
    let (end_line, end_col) = span.end_pos().line_col();

    let delta_line = start_line.saturating_sub(*prev_line);
    let delta_start = if delta_line == 0 {
        start_col.saturating_sub(*prev_col)
    } else {
        start_col - 1
    };

    // обновляем предыдущую позицию
    *prev_line = end_line;
    *prev_col = end_col;

    // рекурсивно строим детей
    let children: Vec<CstNode> = pair
        .clone()
        .into_inner()
        .map(|inner| build_cst(&inner, prev_line, prev_col))
        .collect();

    let kind = map_rule_to_cst_kind(pair.as_rule());
    CstNode {
        kind,
        text: span.as_str().trim().to_string(),
        children,
        start: span.start(),
        end: span.end(),
        delta_line,
        delta_start,
    }
}

fn map_rule_to_cst_kind(rule: Rule) -> CstKind {
    match rule {
        Rule::ident => CstKind::Ident,
        Rule::number => CstKind::Number,
        Rule::string => CstKind::String,
        Rule::boolean => CstKind::Boolean,
        Rule::base_types => CstKind::BaseType,

        Rule::directive => CstKind::Directive,
        Rule::STRUCT | Rule::ENUM | Rule::ELEMENT | Rule::EXPRESSION | Rule::GROUP => CstKind::Keyword,
        Rule::COMMENT_MULTI | Rule::COMMENT_LINE => CstKind::Comment,
        Rule::WHITESPACE => CstKind::Whitespace,
        // составные правила
        Rule::file => CstKind::File,
        Rule::r#struct => CstKind::Struct,
        Rule::r#enum => CstKind::Enum,
        Rule::element => CstKind::Element,
        Rule::block => CstKind::Block,
        Rule::simple_field => CstKind::SimpleField,
        Rule::simple_fields => CstKind::SimpleFields,
        Rule::annotation => CstKind::Annotation,
        Rule::attribute => CstKind::Attribute,
        Rule::attribute_list => CstKind::AttributeList,
        Rule::generic_type => CstKind::GenericType,
        Rule::ns_ident => CstKind::NsIdent,
        Rule::group_content => CstKind::GroupContent,
        Rule::group => CstKind::Group,
        Rule::extend_group => CstKind::ExtendGroup,
        Rule::expression => CstKind::Expression,
        Rule::count => CstKind::Count,
        Rule::group_entry => CstKind::GroupEntry,
        Rule::custom_types => CstKind::CustomType,
        _ => CstKind::Symbol, // fallback
    }
}
