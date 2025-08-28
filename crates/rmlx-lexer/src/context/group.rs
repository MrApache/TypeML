use std::fmt::Display;

use lexer_utils::{push_and_break, Position, Token, TokenType, KEYWORD_TOKEN, TYPE_TOKEN};
use logos::{Lexer, Logos};

use crate::{Error, SchemaTokens};

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
            GroupToken::Identifier => "identifier",
            GroupToken::LeftSquareBracket => "{",
            GroupToken::RightSquareBracket => "}",
            GroupToken::Semicolon => ";",
            GroupToken::Comma => ",",
            GroupToken::NewLine => unreachable!(),
            GroupToken::Keyword => unreachable!(),
            GroupToken::Whitespace => unreachable!(),
        };

        write!(f, "{str}")
    }
}

impl TokenType for GroupToken {
    fn get_token_type(&self) -> u32 {
        match self {
            GroupToken::Keyword => KEYWORD_TOKEN,
            GroupToken::Identifier => TYPE_TOKEN,
            GroupToken::LeftSquareBracket => KEYWORD_TOKEN,
            GroupToken::RightSquareBracket => KEYWORD_TOKEN,
            GroupToken::NewLine => u32::MAX,
            GroupToken::Semicolon => u32::MAX,
            GroupToken::Comma => u32::MAX,
            GroupToken::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn group_callback(
    lex: &mut Lexer<SchemaTokens>,
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
