use logos::{Lexer, Logos};

use crate::{context::attribute::AttributeContext, Position, Token};

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

pub(crate) fn struct_context_callback(lex: &mut Lexer<AttributeContext>) -> Option<Vec<Token<StructContext>>> {
    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: StructContext::Start,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    let mut inner = lex.clone().morph::<StructContext>();
    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                match kind {
                    StructContext::End => break,
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
