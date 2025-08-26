use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::Error;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum AttributeTokens {
    Hash,

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    CloseSquareBracket,

    #[token(",")]
    Comma,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("(", content_callback)]
    Content(Vec<Token<Content>>),
}

impl TokenType for AttributeTokens {
    fn get_token_type(&self) -> u32 {
        match self {
            AttributeTokens::Hash => MACRO,
            AttributeTokens::OpenSquareBracket => MACRO,
            AttributeTokens::CloseSquareBracket => MACRO,
            AttributeTokens::Comma => u32::MAX,
            AttributeTokens::Identifier => MACRO,
            AttributeTokens::NewLine => MACRO,
            AttributeTokens::Whitespace => MACRO,
            AttributeTokens::Content(_) => unreachable!(),
        }
    }
}

pub(crate) fn attribute_callback<'source, T>(
    lex: &mut Lexer<'source, T>,
) -> Result<Vec<Token<AttributeTokens>>, Error>
where
    T: Logos<'source, Extras = Position, Source = str>,
    T: Clone,
{
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, AttributeTokens::Hash, lex);

    let mut inner = lex.clone().morph::<AttributeTokens>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            AttributeTokens::NewLine => inner.extras.new_line(),
            AttributeTokens::Whitespace => inner.extras.current_column += inner.span().len() as u32,
            AttributeTokens::CloseSquareBracket => push_and_break!(&mut tokens, kind, &mut inner),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum Content {
    OpenParenthesis,

    #[regex(r"[^\n)]+", priority = 0)]
    Value,

    #[regex(r#""[^\n"]+""#, priority = 1)]
    String,

    #[token(")")]
    CloseParenthesis,

    #[token("\n")]
    NewLine,
}

impl TokenType for Content {
    fn get_token_type(&self) -> u32 {
        match self {
            Content::Value => u32::MAX,
            Content::String => STRING,
            _ => MACRO,
        }
    }
}

fn content_callback(lex: &mut Lexer<AttributeTokens>) -> Result<Vec<Token<Content>>, Error> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, Content::OpenParenthesis, lex);

    let mut inner = lex.clone().morph::<Content>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            Content::NewLine => inner.extras.new_line(),
            Content::CloseParenthesis => push_and_break!(&mut tokens, kind, &mut inner),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}
