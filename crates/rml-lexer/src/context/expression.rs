use logos::Logos;

use crate::{context::attribute::AttributeContext, Position, Token};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum ExpressionContext {
    #[token("{")]
    Start,

    #[token("}")]
    End,

    #[token(":")]
    Color,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}


pub(crate) fn expression_context_callback(lex: &mut logos::Lexer<AttributeContext>) -> Option<Vec<Token<ExpressionContext>>> {
    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: ExpressionContext::Start,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    let mut inner = lex.clone().morph::<ExpressionContext>();
    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                match kind {
                    ExpressionContext::End => break,
                    ExpressionContext::NewLine => {
                        inner.extras.new_line();
                        continue;
                    },
                    ExpressionContext::Whitespace => {
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

