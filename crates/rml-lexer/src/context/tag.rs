use logos::{Lexer, Logos};
use lexer_utils::*;

use crate::{
    context::attribute::{attribute_context_callback, AttributeContext},
    MarkupTokens,
};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum TagContext {
    #[token("<")]
    TagStart,

    #[token("/")]
    Slash,

    #[token(">")]
    TagEnd,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("=", attribute_context_callback)]
    Attribute(Vec<Token<AttributeContext>>),
}

pub(crate) fn tag_context_callback(
    lex: &mut Lexer<MarkupTokens>,
) -> Option<Vec<Token<TagContext>>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, TagContext::TagStart, lex);

    let mut inner = lex.clone().morph::<TagContext>();
    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            TagContext::TagEnd => push_and_break!(&mut tokens, kind, &mut inner),
            TagContext::NewLine => inner.extras.new_line(),
            TagContext::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
