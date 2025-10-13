use lexer_core::{
    COMMENT_TOKEN, KEYWORD_TOKEN, MACRO_TOKEN, NUMBER_TOKEN, OPERATOR_TOKEN, PARAMETER_TOKEN, STRING_TOKEN, TYPE_TOKEN,
};
use rmlx::{CstKind, CstNode};
use tower_lsp::lsp_types::SemanticToken;

// directive_content = @{ (!">" ~ ANY)* }
// directive = HASH ~ ident ~ (LT ~ directive_content ~ GT)?
fn directive_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::Hash | CstKind::Ident => MACRO_TOKEN,
            CstKind::GT | CstKind::LT => OPERATOR_TOKEN,
            CstKind::DirectiveContent => STRING_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//attribute_list = { HASH ~ LBRACK ~ attribute ~ (COMMA ~ attribute)* ~ RBRACK }
fn attribute_list_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    tokens.push(SemanticToken {
        delta_line: ancestor.delta_line,
        delta_start: ancestor.delta_start,
        length: 1,
        token_type: MACRO_TOKEN,
        token_modifiers_bitset: 0,
    });

    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::Symbol => MACRO_TOKEN,
            CstKind::Hash => return,
            CstKind::Attribute => {
                attribute_tokens(f, tokens);
                return;
            }
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//attribute = { ident ~ (LPAREN ~ base_types ~ RPAREN)? }
fn attribute_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();

    tokens.push(SemanticToken {
        delta_line: cst.delta_line,
        delta_start: cst.delta_start,
        length: iter.next().unwrap().text.len() as u32,
        token_type: MACRO_TOKEN,
        token_modifiers_bitset: 0,
    });

    iter.for_each(|f| {
        let token_type = match f.kind {
            //CstKind::Ident => return,
            CstKind::Symbol => u32::MAX,
            CstKind::BaseType => {
                base_type_tokens(f, tokens);
                return;
            }
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//ident      = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
//string     = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }
//number     = @{ ASCII_DIGIT+ }
//boolean    = { "true" | "false" }
//base_types = { number | boolean | ident | string }
fn base_type_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let token = cst.children.first().unwrap();
    let token_type = match token.kind {
        CstKind::Boolean | CstKind::Number => NUMBER_TOKEN,
        CstKind::Ident => u32::MAX,
        CstKind::String => STRING_TOKEN,
        _ => unreachable!(),
    };

    tokens.push(SemanticToken {
        delta_line: cst.delta_line,
        delta_start: cst.delta_start,
        length: token.text.len() as u32,
        token_type,
        token_modifiers_bitset: 0,
    });
}

//generic_type = { ident ~ LT ~ ns_ident ~ GT }
fn generic_type_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::Ident | CstKind::NsIdent => TYPE_TOKEN,
            CstKind::GT | CstKind::LT => OPERATOR_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//array = { LBRACK ~ ns_ident ~ (COMMA ~ ns_ident)* ~ RBRACK }
fn array_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    tokens.push(SemanticToken {
        delta_line: ancestor.delta_line,
        delta_start: ancestor.delta_start,
        length: 1,
        token_type: u32::MAX,
        token_modifiers_bitset: 0,
    });

    cst.children.iter().skip(1).for_each(|f| {
        let token_type = match f.kind {
            CstKind::Symbol => u32::MAX,
            CstKind::NsIdent => TYPE_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//annotation_value  = { string | array }
fn annotation_value_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let token = cst.children.first().unwrap();
    let token_type = match token.kind {
        CstKind::String => STRING_TOKEN,
        CstKind::Array => {
            array_tokens(cst, token, tokens);
            return;
        }
        _ => unreachable!(),
    };

    tokens.push(SemanticToken {
        delta_line: cst.delta_line,
        delta_start: cst.delta_start,
        length: token.text.len() as u32,
        token_type,
        token_modifiers_bitset: 0,
    });
}

//annotation = { AT ~ ident ~ annotation_value? }
fn annotation_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();
    let token = iter.next().unwrap();
    assert_eq!(token.kind, CstKind::AT);
    tokens.push(SemanticToken {
        delta_line: ancestor.delta_line,
        delta_start: ancestor.delta_start,
        length: token.text.len() as u32,
        token_type: MACRO_TOKEN,
        token_modifiers_bitset: 0,
    });

    iter.for_each(|f| {
        let token_type = match f.kind {
            CstKind::Ident => MACRO_TOKEN,
            CstKind::AnnotationValue => {
                annotation_value_tokens(f, tokens);
                return;
            }
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//simple_field = { annotation* ~ ident ~ COLON ~ (generic_type | ns_ident) }
fn simple_field(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();
    let token = iter.next().unwrap();
    match token.kind {
        CstKind::Annotation => annotation_tokens(ancestor, token, tokens),
        CstKind::Ident => tokens.push(SemanticToken {
            delta_line: ancestor.delta_line,
            delta_start: ancestor.delta_start,
            length: token.text.len() as u32,
            token_type: PARAMETER_TOKEN,
            token_modifiers_bitset: 0,
        }),
        _ => unreachable!(),
    }

    iter.for_each(|f| {
        let token_type = match f.kind {
            CstKind::GenericType => {
                generic_type_tokens(f, tokens);
                return;
            }
            CstKind::Symbol => u32::MAX,
            CstKind::NsIdent => TYPE_TOKEN,
            CstKind::Ident => PARAMETER_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//simple_fields = { simple_field ~ (COMMA ~ simple_field?)* ~ COMMA? }
fn simple_fields(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::SimpleField => {
                simple_field(cst, f, tokens);
                return;
            }
            CstKind::Symbol => u32::MAX,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//block = { LBRACE ~ simple_fields ~ RBRACE }
fn block_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    //let mut iter = cst.children.iter();

    tokens.push(SemanticToken {
        delta_line: cst.delta_line,
        delta_start: cst.delta_start,
        length: 1,
        token_type: u32::MAX,
        token_modifiers_bitset: 0,
    });

    cst.children.iter().skip(1).for_each(|f| {
        let token_type = match f.kind {
            CstKind::Symbol => u32::MAX,
            CstKind::SimpleFields => {
                simple_fields(f, tokens);
                return;
            }
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//struct = { attribute_list* ~ STRUCT ~ (generic_type | ident) ~ block }
fn struct_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::AttributeList => {
                attribute_list_tokens(ancestor, f, tokens);
                return;
            }
            CstKind::GenericType => {
                generic_type_tokens(f, tokens);
                return;
            }
            CstKind::Block => {
                block_tokens(f, tokens);
                return;
            }
            CstKind::Ident => TYPE_TOKEN,
            CstKind::Keyword => KEYWORD_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//element = { attribute_list* ~ ELEMENT ~ ident ~ ARROW ~ ns_ident ~ (block | SEMI) }
fn element_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();
    let token = iter.next().unwrap();
    match token.kind {
        CstKind::AttributeList => attribute_list_tokens(ancestor, token, tokens),
        CstKind::Keyword => tokens.push(SemanticToken {
            delta_line: ancestor.delta_line,
            delta_start: ancestor.delta_start,
            length: token.text.len() as u32,
            token_type: KEYWORD_TOKEN,
            token_modifiers_bitset: 0,
        }),
        _ => unreachable!(),
    }

    iter.for_each(|f| {
        let token_type = match f.kind {
            CstKind::Block => {
                block_tokens(f, tokens);
                return;
            }
            CstKind::Keyword => KEYWORD_TOKEN,
            CstKind::Arrow => OPERATOR_TOKEN,
            CstKind::Ident | CstKind::NsIdent => TYPE_TOKEN,
            CstKind::Symbol => u32::MAX,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//enum_variant = { annotation* ~ ((ident ~ LPAREN ~ ns_ident ~ RPAREN) | ident) }
fn enum_variant_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::Annotation => {
                annotation_tokens(cst, f, tokens); //TODO fix
                return;
            }
            CstKind::Ident => PARAMETER_TOKEN,
            CstKind::Symbol => u32::MAX,
            CstKind::NsIdent => TYPE_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//enum = { attribute_list* ~ ENUM ~ ident ~ LBRACE ~ enum_variant ~ (COMMA ~ enum_variant)* ~ COMMA? ~ RBRACE }
fn enum_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::AttributeList => {
                attribute_list_tokens(ancestor, f, tokens);
                return;
            }
            CstKind::EnumVariant => {
                enum_variant_tokens(f, tokens);
                return;
            }
            CstKind::Keyword => KEYWORD_TOKEN,
            CstKind::Ident => TYPE_TOKEN,
            CstKind::Symbol => u32::MAX,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//count = { LPAREN ~ ((number ~ DASH ~ number) | number | STAR | QMARK | PLUS) ~ RPAREN }
fn count_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();
    let token = iter.next().unwrap();
    assert_eq!(token.kind, CstKind::Symbol);
    tokens.push(SemanticToken {
        delta_line: cst.delta_line,
        delta_start: cst.delta_start,
        length: 1,
        token_type: OPERATOR_TOKEN,
        token_modifiers_bitset: 0,
    });

    iter.for_each(|f| {
        let token_type = match f.kind {
            CstKind::Symbol => OPERATOR_TOKEN,
            CstKind::Number => NUMBER_TOKEN,
            CstKind::Star | CstKind::QMark | CstKind::Plus => MACRO_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//group_entry  = { PLUS ~ UNIQUE? ~ ns_ident ~ count }
fn group_entry_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();
    let token = iter.next().unwrap();
    assert_eq!(token.kind, CstKind::Plus);
    tokens.push(SemanticToken {
        delta_line: cst.delta_line,
        delta_start: cst.delta_start,
        length: token.text.len() as u32,
        token_type: OPERATOR_TOKEN,
        token_modifiers_bitset: 0,
    });

    iter.for_each(|f| {
        let token_type = match f.kind {
            CstKind::Keyword => KEYWORD_TOKEN,
            CstKind::NsIdent => TYPE_TOKEN,
            CstKind::Count => {
                count_tokens(f, tokens);
                return;
            }
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//group_content = { LBRACE ~ group_entry* ~ RBRACE }
fn group_content_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();
    let token = iter.next().unwrap();
    assert_eq!(token.kind, CstKind::Symbol);
    tokens.push(SemanticToken {
        delta_line: cst.delta_line,
        delta_start: cst.delta_start,
        length: token.text.len() as u32,
        token_type: u32::MAX,
        token_modifiers_bitset: 0,
    });

    iter.for_each(|f| {
        let token_type = match f.kind {
            CstKind::Symbol => u32::MAX,
            CstKind::GroupEntry => {
                group_entry_tokens(f, tokens);
                return;
            }
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//group = { attribute_list* ~ GROUP ~ ident ~ count* ~ (group_content | SEMI) }
fn group_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();

    let token = iter.next().unwrap();
    match token.kind {
        CstKind::AttributeList => attribute_list_tokens(ancestor, token, tokens),
        CstKind::Keyword => tokens.push(
            SemanticToken {
                delta_line: ancestor.delta_line,
                delta_start: ancestor.delta_start,
                length: token.text.len() as u32,
                token_type: KEYWORD_TOKEN,
                token_modifiers_bitset: 0,
            }),
        _ => unreachable!()
    }

    iter.for_each(|f| {
        let token_type = match f.kind {
            CstKind::Count => {
                count_tokens(f, tokens);
                return;
            }
            CstKind::GroupContent => {
                group_content_tokens(f, tokens);
                return;
            }
            CstKind::Keyword => KEYWORD_TOKEN,
            CstKind::Ident => TYPE_TOKEN,
            CstKind::Symbol => u32::MAX,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//expression = { (attribute_list | annotation)* ~ EXPRESSION ~ ident ~ block }
fn expression_tokens(ancestor: &CstNode, cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let mut iter = cst.children.iter();
    let token = iter.next().unwrap();
    match token.kind {
        CstKind::AttributeList => attribute_list_tokens(ancestor, token, tokens),
        CstKind::Annotation => annotation_tokens(ancestor, token, tokens),
        CstKind::Keyword => tokens.push(SemanticToken {
            delta_line: ancestor.delta_line,
            delta_start: ancestor.delta_start,
            length: token.text.len() as u32,
            token_type: KEYWORD_TOKEN,
            token_modifiers_bitset: 0,
        }),
        _ => unreachable!()
    }

    cst.children.iter().for_each(|f| {
        let token_type = match f.kind {
            CstKind::AttributeList => {
                attribute_list_tokens(f, f, tokens);
                return;
            }
            CstKind::Annotation => {
                annotation_tokens(f, f, tokens);
                return;
            }
            CstKind::Block => {
                block_tokens(f, tokens);
                return;
            }
            CstKind::Keyword => KEYWORD_TOKEN,
            CstKind::Ident => TYPE_TOKEN,
            _ => unreachable!(),
        };

        tokens.push(SemanticToken {
            delta_line: f.delta_line,
            delta_start: f.delta_start,
            length: f.text.len() as u32,
            token_type,
            token_modifiers_bitset: 0,
        });
    });
}

//custom_types = { enum | struct | element | extend_group | group | expression }
fn custom_type_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    let token = cst.children.first().unwrap();
    match token.kind {
        CstKind::Struct => struct_tokens(cst, token, tokens),
        CstKind::Element => element_tokens(cst, token, tokens),
        CstKind::Enum => enum_tokens(cst, token, tokens),
        CstKind::Expression => expression_tokens(cst, token, tokens),
        CstKind::Group | CstKind::ExtendGroup => group_tokens(cst, token, tokens),
        _ => unreachable!(),
    }
}

fn file_tokens(cst: &CstNode, tokens: &mut Vec<SemanticToken>) {
    cst.children.iter().for_each(|f| match f.kind {
        CstKind::Comment => {
            tokens.push(SemanticToken {
                delta_line: f.delta_line,
                delta_start: f.delta_start,
                length: f.text.len() as u32,
                token_type: COMMENT_TOKEN,
                token_modifiers_bitset: 0,
            });
        }
        CstKind::Directive => directive_tokens(f, tokens),
        CstKind::CustomType => custom_type_tokens(f, tokens),
        CstKind::Symbol => {}
        _ => unreachable!("{f:#?}"),
    });
}

pub fn get_tokens(cst: &CstNode) -> Vec<SemanticToken> {
    assert!(matches!(cst.kind, CstKind::File));
    let mut tokens = vec![];
    file_tokens(cst, &mut tokens);
    tokens
}
