use logos::{Lexer, Logos};

use crate::{
    context::attribute::{attribute_context_callback, AttributeContext},
    DefaultContext, Position, Token,
};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum TagContext {
    #[token(">")]
    TagEnd,

    #[token("/>")]
    TagCloseEnd,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("=")]
    AttributeEqual,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("\"", attribute_context_callback)]
    Attribute(Vec<Token<AttributeContext>>),
}

pub(crate) fn tag_context_callback(lex: &mut Lexer<DefaultContext>) -> Option<Vec<Token<TagContext>>> {
    let mut inner = lex.clone().morph::<TagContext>();
    let mut tokens = Vec::new();
    let mut delta_start = 1;

    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                if matches!(kind, TagContext::TagEnd | TagContext::TagCloseEnd) {
                    break;
                }

                if kind == TagContext::NewLine {
                    inner.extras.current_line += 1;
                    inner.extras.previous_token_end_column = 0;
                    inner.extras.current_column = 0;
                    delta_start = 0;
                    continue;
                }

                if kind == TagContext::Whitespace {
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
                inner.extras.current_column = delta_start as u32;
            }
            Err(_) => return None,
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
