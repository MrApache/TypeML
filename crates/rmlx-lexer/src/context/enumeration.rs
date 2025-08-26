use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{attribute_context_callback_enum, AttributeContext, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum EnumContext {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token("(")]
    LeftParenthesis,

    #[token(")")]
    RightParenthesis,

    #[token("\n")]
    NewLine,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("#", attribute_context_callback_enum)]
    Attribute(Vec<Token<AttributeContext>>),
}

impl TokenType for EnumContext {
    fn get_token_type(&self) -> u32 {
        match self {
            EnumContext::Keyword => KEYWORD,
            EnumContext::Identifier => TYPE,
            EnumContext::LeftCurlyBracket => u32::MAX,
            EnumContext::RightCurlyBracket => u32::MAX,
            EnumContext::LeftParenthesis => u32::MAX,
            EnumContext::RightParenthesis => u32::MAX,
            EnumContext::NewLine => u32::MAX,
            EnumContext::Comma => u32::MAX,
            EnumContext::Whitespace => u32::MAX,
            EnumContext::Attribute(_) => unreachable!()
        }
    }
}

pub(crate) fn enum_context_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Option<Vec<Token<EnumContext>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: EnumContext::Keyword,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip 'enum'
    lex.extras.current_column += 4;

    let mut inner = lex.clone().morph::<EnumContext>();

    let mut bracket_depth = 0;

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            EnumContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            EnumContext::Whitespace => {
                inner.extras.current_column += inner.span().len() as u32;
                continue;
            }
            _ => {
                if let EnumContext::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let EnumContext::RightCurlyBracket = &kind {
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

