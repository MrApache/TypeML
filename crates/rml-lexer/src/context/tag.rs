use logos::{Lexer, Logos};

use crate::{
    context::attribute::{attribute_context_callback, AttributeContext},
    DefaultContext, Position, Token,
};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum TagContext {
    #[token("<")]
    TagStart,

    #[token("</")]
    TagCloseStart,

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
    let mut tokens = Vec::new();
    let kind = match lex.span().len() {
        1 => TagContext::TagStart,
        2 => TagContext::TagCloseStart,
        _ => unreachable!()
    };

    tokens.push(Token {
        kind,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    let mut inner = lex.clone().morph::<TagContext>();

    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                if matches!(kind, TagContext::TagEnd | TagContext::TagCloseEnd) {
                    break;
                }

                if kind == TagContext::NewLine {
                    inner.extras.new_line();
                    continue;
                }

                if kind == TagContext::Whitespace {
                    inner.extras.current_column += inner.span().len() as u32;
                    continue;
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
