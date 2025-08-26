use lexer_utils::*;
use logos::{Lexer, Logos};

use crate::SchemaTokens;


#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum ElementContext {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("[")]
    LeftSquareBracket,

    #[token("]")]
    RightSquareBracket,

    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token("<")]
    LeftAngleBracket,

    #[token(">")]
    RightAngleBracket,

    #[token("\n")]
    NewLine,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for ElementContext {
    fn get_token_type(&self) -> u32 {
        match self {
            ElementContext::Keyword => KEYWORD,
            ElementContext::Identifier => TYPE,
            ElementContext::LeftSquareBracket => KEYWORD,
            ElementContext::RightSquareBracket => KEYWORD,
            ElementContext::LeftCurlyBracket => u32::MAX,
            ElementContext::RightCurlyBracket => u32::MAX,
            ElementContext::LeftAngleBracket => OPERATOR,
            ElementContext::RightAngleBracket => OPERATOR,
            ElementContext::NewLine => u32::MAX,
            ElementContext::Colon => u32::MAX,
            ElementContext::Semicolon => u32::MAX,
            ElementContext::Comma => u32::MAX,
            ElementContext::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn element_context_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Option<Vec<Token<ElementContext>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: ElementContext::Keyword,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip 'element'
    lex.extras.current_column += 7;

    let mut inner = lex.clone().morph::<ElementContext>();

    let mut bracket_depth = 0;

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            ElementContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            ElementContext::Semicolon => {
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });

                inner.extras.current_column += inner.span().len() as u32;
                break;
            },
            ElementContext::Whitespace => {
                inner.extras.current_column += inner.span().len() as u32;
                continue;
            }
            _ => {
                if let ElementContext::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let ElementContext::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        panic!();
                    }
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        tokens.push(Token {
                            kind,
                            span: inner.span(),
                            delta_line: inner.extras.get_delta_line(),
                            delta_start: inner.extras.get_delta_start(),
                        });

                        inner.extras.current_column += inner.span().len() as u32;
                        break;
                    }
                }
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });

                inner.extras.current_column += inner.span().len() as u32;
            }
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
