
use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::SchemaTokens;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum StructContext {
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

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for StructContext {
    fn get_token_type(&self) -> u32 {
        match self {
            StructContext::Keyword => KEYWORD,
            StructContext::Identifier => TYPE,
            StructContext::LeftSquareBracket => KEYWORD,
            StructContext::RightSquareBracket => KEYWORD,
            StructContext::LeftCurlyBracket => u32::MAX,
            StructContext::RightCurlyBracket => u32::MAX,
            StructContext::LeftAngleBracket => OPERATOR,
            StructContext::RightAngleBracket => OPERATOR,
            StructContext::NewLine => u32::MAX,
            StructContext::Colon => u32::MAX,
            StructContext::Comma => u32::MAX,
            StructContext::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn struct_context_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Option<Vec<Token<StructContext>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: StructContext::Keyword,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip 'struct'
    lex.extras.current_column += 6;

    let mut inner = lex.clone().morph::<StructContext>();

    let mut bracket_depth = 0;

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            StructContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            StructContext::Whitespace => {
                inner.extras.current_column += inner.span().len() as u32;
                continue;
            }
            _ => {
                if let StructContext::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let StructContext::RightCurlyBracket = &kind {
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

