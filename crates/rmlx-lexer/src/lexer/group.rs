use crate::{Error, NamedStatement, SchemaStatement, StatementTokens, TokenArrayProvider};
use lexer_utils::{push_and_break, unwrap_or_continue, Position, Token};
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum GroupDefinitionToken {
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

    SyntaxError,
}

impl Display for GroupDefinitionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GroupDefinitionToken::Keyword => "group",
            GroupDefinitionToken::Identifier => "identifier",
            GroupDefinitionToken::LeftSquareBracket => "{",
            GroupDefinitionToken::RightSquareBracket => "}",
            GroupDefinitionToken::Semicolon => ";",
            GroupDefinitionToken::Comma => ",",
            GroupDefinitionToken::NewLine => unreachable!(),
            GroupDefinitionToken::Whitespace => unreachable!(),
            GroupDefinitionToken::SyntaxError => "error",
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for GroupDefinitionToken {
    fn keyword() -> &'static str {
        "group"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl NamedStatement for GroupDefinitionToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

impl TokenArrayProvider for GroupDefinitionToken {
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

pub(crate) fn group_callback(lex: &mut Lexer<SchemaStatement>) -> Vec<Token<GroupDefinitionToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, GroupDefinitionToken::Keyword, lex);

    let mut inner = lex.clone().morph::<GroupDefinitionToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, GroupDefinitionToken::SyntaxError, &mut inner) {
            GroupDefinitionToken::NewLine => inner.extras.new_line(),
            GroupDefinitionToken::Semicolon => push_and_break!(&mut tokens, GroupDefinitionToken::Semicolon, &mut inner),
            GroupDefinitionToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    tokens
}
