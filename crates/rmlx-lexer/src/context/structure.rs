use std::fmt::Display;

use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, SchemaTokens, TokenDefinition};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum StructToken {
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

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenDefinition for StructToken {
    fn keyword() -> &'static str {
        "struct"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }

    fn left_curly_brace() -> Self {
        Self::LeftCurlyBracket
    }

    fn right_curly_brace() -> Self {
        Self::RightCurlyBracket
    }

    fn identifier() -> Self {
        Self::Identifier
    }

    fn colon() -> Self {
        Self::Colon
    }
}

impl Display for StructToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StructToken::Keyword => "struct",
            StructToken::Identifier => "identifier",
            StructToken::LeftCurlyBracket => "{",
            StructToken::RightCurlyBracket => "}",
            StructToken::LeftAngleBracket => "<",
            StructToken::RightAngleBracket => ">",
            StructToken::Colon => ";",
            StructToken::Comma => ",",
            StructToken::NewLine => unreachable!(),
            StructToken::Whitespace => unreachable!(),
        };
        write!(f, "{str}")
    }
}

impl TokenType for StructToken {
    fn get_token_type(&self) -> u32 {
        match self {
            StructToken::Keyword => KEYWORD_TOKEN,
            StructToken::Identifier => TYPE_TOKEN,
            StructToken::LeftCurlyBracket => u32::MAX,
            StructToken::RightCurlyBracket => u32::MAX,
            StructToken::LeftAngleBracket => OPERATOR_TOKEN,
            StructToken::RightAngleBracket => OPERATOR_TOKEN,
            StructToken::NewLine => u32::MAX,
            StructToken::Colon => u32::MAX,
            StructToken::Comma => u32::MAX,
            StructToken::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn struct_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<StructToken>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, StructToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<StructToken>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            StructToken::NewLine => inner.extras.new_line(),
            StructToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => {
                if let StructToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let StructToken::RightCurlyBracket = &kind {
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

