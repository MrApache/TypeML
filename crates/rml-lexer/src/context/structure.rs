use logos::{Lexer, Logos};

use crate::{context::attribute::AttributeContext, Position, Token, TokenType};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum StructContext {
    #[token("{{")]
    Start,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token(":")]
    Assing,

    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    #[regex(r"[0-9]+")]
    Int,

    #[token("}}")]
    End,

    #[token(",")]
    Comma,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for StructContext {
    fn get_token_type(&self) -> u32 {
        match self {
            StructContext::Start => 7,
            StructContext::End => 7,

            StructContext::Identifier => 1,
            StructContext::Float => 5,
            StructContext::Int => 5,

            StructContext::Assing => u32::MAX,
            StructContext::Comma => u32::MAX,
            StructContext::NewLine => u32::MAX,
            StructContext::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn struct_context_callback(lex: &mut Lexer<AttributeContext>) -> Option<Vec<Token<StructContext>>> {
    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: StructContext::Start,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '{{'
    lex.extras.current_column += 2;

    let mut inner = lex.clone().morph::<StructContext>();
    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                match kind {
                    StructContext::End => {
                        tokens.push(Token {
                            kind,
                            span: inner.span(),
                            delta_line: inner.extras.get_delta_line(),
                            delta_start: inner.extras.get_delta_start(),
                        });
                        inner.extras.current_column += inner.span().len() as u32;
                        break;
                    },
                    StructContext::NewLine => {
                        inner.extras.new_line();
                        continue;
                    },
                    StructContext::Whitespace => {
                        inner.extras.current_column += inner.span().len() as u32;
                        continue;
                    },
                    _ => {},
                }

                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });
                inner.extras.current_column += inner.span().len() as u32;
            }
            Err(_) => return None,
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
