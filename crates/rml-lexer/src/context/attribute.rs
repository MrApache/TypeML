use logos::{Lexer, Logos};

use crate::{
    context::{
        expression::ExpressionContext, expression_context_callback, struct_context_callback,
        structure::StructContext, tag::TagContext,
    },
    Position, Token,
};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum AttributeContext {
    #[token("\"")]
    Quote,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Value,

    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    #[regex(r"[0-9]+")]
    Int,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("{", expression_context_callback)]
    Expression(Vec<Token<ExpressionContext>>),

    #[token("{{", struct_context_callback)]
    Struct(Vec<Token<StructContext>>),
}

pub(crate) fn attribute_context_callback(
    lex: &mut Lexer<TagContext>,
) -> Option<Vec<Token<AttributeContext>>> {
    let mut tokens = Vec::new();

    let mut inner = lex.clone().morph::<AttributeContext>();
    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                match kind {
                    AttributeContext::Quote => break,
                    AttributeContext::NewLine => {
                        inner.extras.new_line();
                        continue;
                    },
                    AttributeContext::Whitespace => {
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
