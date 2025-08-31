use std::fmt::Display;

use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, StatementTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum AttributeToken {
    Hash,

    #[token("[")]
    LeftSquareBracket,

    #[token("]")]
    RightSquareBracket,

    #[token(",")]
    Comma,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("(", content_callback)]
    Content(Vec<Token<ContentToken>>),
}

impl Display for AttributeToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AttributeToken::Content(_) => "content",
            AttributeToken::Identifier => "identifier",
            AttributeToken::Hash => "#",
            AttributeToken::LeftSquareBracket => "[",
            AttributeToken::RightSquareBracket => "]",
            AttributeToken::Comma => ",",
            AttributeToken::NewLine => unreachable!(),
            AttributeToken::Whitespace => unreachable!(),
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for AttributeToken {
    fn keyword() -> &'static str {
        "#"
    }

    fn keyword_token() -> Self {
        Self::Hash
    }
}

pub(crate) fn attribute_callback<'source, T>(
    lex: &mut Lexer<'source, T>,
) -> Result<Vec<Token<AttributeToken>>, Error>
where
    T: Logos<'source, Extras = Position, Source = str>,
    T: Clone,
{
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, AttributeToken::Hash, lex);

    let mut inner = lex.clone().morph::<AttributeToken>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            AttributeToken::NewLine => inner.extras.new_line(),
            AttributeToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            AttributeToken::RightSquareBracket => push_and_break!(&mut tokens, kind, &mut inner),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum ContentToken {
    LeftParenthesis,

    #[regex(r"[^\n)]+", priority = 0)]
    Value,

    #[regex(r#""[^\n"]+""#, priority = 1)]
    String,

    #[token(")")]
    RightParenthesis,

    #[token("\n")]
    NewLine,
}

impl Display for ContentToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ContentToken::Value => "value",
            ContentToken::String => "string",
            ContentToken::LeftParenthesis => "(",
            ContentToken::RightParenthesis => ")",
            ContentToken::NewLine => unreachable!(),
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for ContentToken {
    fn keyword() -> &'static str {
        "("
    }

    fn keyword_token() -> Self {
        Self::LeftParenthesis
    }
}

fn content_callback(lex: &mut Lexer<AttributeToken>) -> Result<Vec<Token<ContentToken>>, Error> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, ContentToken::LeftParenthesis, lex);

    let mut inner = lex.clone().morph::<ContentToken>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            ContentToken::NewLine => inner.extras.new_line(),
            ContentToken::RightParenthesis => push_and_break!(&mut tokens, kind, &mut inner),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}
