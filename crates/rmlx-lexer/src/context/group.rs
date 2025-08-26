use lexer_utils::{push_and_break, Position, Token, TokenType, KEYWORD, TYPE};
use logos::{Lexer, Logos};

use crate::{Error, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum GroupTokens {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("[")]
    ArrayOpen,

    #[token("]")]
    ArrayClose,

    #[token("\n")]
    NewLine,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for GroupTokens {
    fn get_token_type(&self) -> u32 {
        match self {
            GroupTokens::Keyword => KEYWORD,
            GroupTokens::Identifier => TYPE,
            GroupTokens::ArrayOpen => KEYWORD,
            GroupTokens::ArrayClose => KEYWORD,
            GroupTokens::NewLine => u32::MAX,
            GroupTokens::Semicolon => u32::MAX,
            GroupTokens::Comma => u32::MAX,
            GroupTokens::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn group_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<GroupTokens>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, GroupTokens::Keyword, lex);

    let mut inner = lex.clone().morph::<GroupTokens>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            GroupTokens::NewLine => inner.extras.new_line(),
            GroupTokens::Semicolon => push_and_break!(&mut tokens, kind, &mut inner),
            GroupTokens::Whitespace => inner.extras.current_column += inner.span().len() as u32,
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}
