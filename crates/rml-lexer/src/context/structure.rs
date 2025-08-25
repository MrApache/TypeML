use logos::{Lexer, Logos};

use crate::{context::attribute::AttributeContext, Position, Token};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum StructContext {
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
    let mut inner = lex.clone().morph::<StructContext>();
    let mut tokens = Vec::new();
    let mut delta_start = 1;

    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                if matches!(kind, StructContext::End) {
                    if kind == StructContext::NewLine {
                        inner.extras.current_line += 1;
                    }
                    break;
                }
                if kind == StructContext::NewLine {
                    inner.extras.current_line += 1;
                    inner.extras.previous_token_end_column = 0;
                    inner.extras.current_column = 0;
                    delta_start = 0;
                    continue;
                }
                if kind == StructContext::Whitespace {
                    delta_start += inner.span().len();
                    continue;
                }

                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: delta_start as u32 - inner.extras.previous_token_end_column,
                    length: inner.span().len() as u32,
                });
                inner.extras.previous_token_end_column = delta_start as u32;
                delta_start += inner.span().len();
                inner.extras.current_line = delta_start as u32;
            }
            Err(_) => return None,
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
