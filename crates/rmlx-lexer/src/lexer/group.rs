use crate::{Error, NamedStatement, SchemaStatement, StatementTokens, TokenArrayProvider};
use lexer_utils::{push_and_break, Position, Token};
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum GroupToken {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("[")]
    LeftSquareBracket,

    #[token("]")]
    RightSquareBracket,

    #[token("\n")]
    NewLine,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl Display for GroupToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GroupToken::Keyword => "group",
            GroupToken::Identifier => "identifier",
            GroupToken::LeftSquareBracket => "{",
            GroupToken::RightSquareBracket => "}",
            GroupToken::Semicolon => ";",
            GroupToken::Comma => ",",
            GroupToken::NewLine => unreachable!(),
            GroupToken::Whitespace => unreachable!(),
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for GroupToken {
    fn keyword() -> &'static str {
        "group"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl NamedStatement for GroupToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

impl TokenArrayProvider for GroupToken {
    fn comma() -> Self {
        Self::Comma
    }

    fn left_square_bracket() -> Self {
        Self::LeftSquareBracket
    }

    fn right_square_bracket() -> Self {
        Self::RightSquareBracket
    }
}

pub(crate) fn group_callback(
    lex: &mut Lexer<SchemaStatement>,
) -> Result<Vec<Token<GroupToken>>, Error> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, GroupToken::Keyword, lex);

    let mut inner = lex.clone().morph::<GroupToken>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            GroupToken::NewLine => inner.extras.new_line(),
            GroupToken::Semicolon => push_and_break!(&mut tokens, kind, &mut inner),
            GroupToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}
