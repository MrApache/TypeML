use crate::pest::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct CstNode {
    pub kind: CstKind,
    pub text: String,
    pub children: Vec<CstNode>,
    pub start: usize,     // абсолютная позиция в файле
    pub end: usize,       // абсолютная позиция в файле
    pub delta_line: u32,  // строки от предыдущего токена
    pub delta_start: u32, // смещение в строке
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    ExtendGroup,
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

pub fn build_cst(
    pair: &Pair<Rule>,
    source: &str,
    prev_line: &mut u32,
    prev_col: &mut u32,
) -> CstNode {
    let span = pair.as_span();
    let (start_line_1, start_col_1) = span.start_pos().line_col();
    let (end_line_1, end_col_1) = span.end_pos().line_col();

    // LSP uses 0-based indexing
    let start_line = (start_line_1 - 1) as u32;
    let end_line = (end_line_1 - 1) as u32;

    // Compute UTF-16 columns
    let byte_start = span.start();
    let byte_end = span.end();

    let line_start_byte = source[..byte_start]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    let start_col_utf16 = source[line_start_byte..byte_start].encode_utf16().count() as u32;
    let end_col_utf16 = source[line_start_byte..byte_end].encode_utf16().count() as u32;

    // --- delta calculation (relative to previous token start) ---
    let delta_line = if start_line > *prev_line {
        start_line - *prev_line
    } else {
        0
    };

    let delta_start = if delta_line == 0 {
        start_col_utf16.saturating_sub(*prev_col)
    } else {
        start_col_utf16
    };

    // Теперь prev обновляем ПОСЛЕ того, как вычислили дельты
    *prev_line = start_line;
    *prev_col = start_col_utf16;

    let kind = map_rule_to_cst_kind(pair.as_rule());
    
    // рекурсивно строим детей
    let children: Vec<CstNode> = pair
        .clone()
        .into_inner()
        .map(|inner| build_cst(&inner, source, prev_line, prev_col))
        .collect();


    CstNode {
        kind,
        text: span.as_str().trim().to_string(),
        children,
        start: byte_start,
        end: byte_end,
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

        Rule::EXTEND
            | Rule::UNIQUE
            | Rule::STRUCT
            | Rule::ENUM
            | Rule::ELEMENT
            | Rule::EXPRESSION
            | Rule::GROUP => CstKind::Keyword,

        Rule::COMMENT_MULTI | Rule::COMMENT_LINE => CstKind::Comment,
        Rule::WHITESPACE => CstKind::Whitespace,

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

        Rule::array => CstKind::Array,
        Rule::LT => CstKind::LT,
        Rule::GT => CstKind::GT,
        Rule::HASH => CstKind::Hash,
        Rule::PLUS => CstKind::Plus,
        Rule::STAR => CstKind::Star,
        Rule::QMARK => CstKind::QMark,
        Rule::ARROW => CstKind::Arrow,
        Rule::AT => CstKind::AT,
        Rule::directive_content => CstKind::DirectiveContent,
        Rule::annotation_value => CstKind::AnnotationValue,
        Rule::enum_variant => CstKind::EnumVariant,
        _ => CstKind::Symbol

    }
}
