use std::fmt::Display;

use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, NamedStatement, SchemaStatement, TokenBodyStatement, StatementTokens, TokenSimpleTypeProvider};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum ElementToken {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token("<")]
    LeftAngleBracket,

    #[token(">")]
    RightAngleBracket,

    #[token("\n")]
    NewLine,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    SyntaxError
}

impl Display for ElementToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ElementToken::Keyword => "element",
            ElementToken::Identifier => "identifier",
            ElementToken::LeftCurlyBracket => "{",
            ElementToken::RightCurlyBracket => "}",
            ElementToken::LeftAngleBracket => "<",
            ElementToken::RightAngleBracket => ">",
            ElementToken::Colon => ":",
            ElementToken::Semicolon => ";",
            ElementToken::Comma => ",",
            ElementToken::NewLine => unreachable!(),
            ElementToken::Whitespace => unreachable!(),
            ElementToken::SyntaxError => "error",
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for ElementToken {
    fn keyword() -> &'static str {
        "element"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl TokenBodyStatement for ElementToken {
    fn left_curly_bracket() -> Self {
        Self::LeftCurlyBracket
    }

    fn right_curly_bracket() -> Self {
        Self::RightCurlyBracket
    }
}

impl NamedStatement for ElementToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

impl TokenSimpleTypeProvider for ElementToken {
    fn colon() -> Self {
        Self::Colon
    }

    fn left_angle_bracket() -> Self {
        Self::LeftAngleBracket
    }

    fn right_angle_bracket() -> Self {
        Self::RightAngleBracket
    }
}

pub(crate) fn element_callback(
    lex: &mut Lexer<SchemaStatement>,
) -> Vec<Token<ElementToken>> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, ElementToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<ElementToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, ElementToken::SyntaxError, &mut inner) {
            ElementToken::NewLine => inner.extras.new_line(),
            ElementToken::Semicolon => push_and_break!(&mut tokens, ElementToken::Semicolon, &mut inner),
            ElementToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => {
                if let ElementToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let ElementToken::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        Token::push_with_advance(&mut tokens, ElementToken::SyntaxError, &mut inner);
                        return tokens;
                    }
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        push_and_break!(&mut tokens, kind, &mut inner);
                    }
                }
                Token::push_with_advance(&mut tokens, kind, &mut inner)
            }
        }
    }

    *lex = inner.morph();
    tokens
}
