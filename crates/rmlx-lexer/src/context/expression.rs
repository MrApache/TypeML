use std::fmt::Display;

use crate::{
    Error, NamedStatement, SchemaStatement, TokenArrayProvider, TokenDefinition,
    TokenSimpleTypeProvider,
};
use lexer_utils::*;
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum ExpressionToken {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("[")]
    LeftSquareBracket,

    #[token("]")]
    RightSquareBracket,

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

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl Display for ExpressionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ExpressionToken::Keyword => "expression",
            ExpressionToken::Identifier => "identifier",
            ExpressionToken::LeftSquareBracket => "]",
            ExpressionToken::RightSquareBracket => "[",
            ExpressionToken::LeftCurlyBracket => "{",
            ExpressionToken::RightCurlyBracket => "}",
            ExpressionToken::LeftAngleBracket => "<",
            ExpressionToken::RightAngleBracket => ">",
            ExpressionToken::Colon => ":",
            ExpressionToken::Comma => ",",
            ExpressionToken::NewLine => unreachable!(),
            ExpressionToken::Whitespace => unreachable!(),
        };

        write!(f, "{str}")
    }
}

impl TokenDefinition for ExpressionToken {
    fn keyword() -> &'static str {
        "expression"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }

    fn colon() -> Self {
        Self::Colon
    }

    fn left_curly_bracket() -> Self {
        Self::LeftCurlyBracket
    }

    fn right_curly_bracket() -> Self {
        Self::RightCurlyBracket
    }
}

impl TokenSimpleTypeProvider for ExpressionToken {
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

impl TokenArrayProvider for ExpressionToken {
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

impl NamedStatement for ExpressionToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

pub(crate) fn expression_callback(
    lex: &mut Lexer<SchemaStatement>,
) -> Result<Vec<Token<ExpressionToken>>, Error> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, ExpressionToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<ExpressionToken>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            ExpressionToken::NewLine => inner.extras.new_line(),
            ExpressionToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => {
                if let ExpressionToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                } else if let ExpressionToken::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        return Err(Error::MissingOpeningBrace);
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
    Ok(tokens)
}
