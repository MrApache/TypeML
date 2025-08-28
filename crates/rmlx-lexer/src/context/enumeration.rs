use std::fmt::Display;

use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{attribute_callback, AttributeToken, Error, NamedStatement, SchemaStatement, TokenDefinition};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum EnumToken {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token("(")]
    LeftParenthesis,

    #[token(")")]
    RightParenthesis,

    #[token("\n")]
    NewLine,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("#", attribute_callback)]
    Attribute(Vec<Token<AttributeToken>>),
}

impl Display for EnumToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EnumToken::Keyword => "enum",
            EnumToken::Identifier => "identifier",
            EnumToken::LeftCurlyBracket => "{",
            EnumToken::RightCurlyBracket => "}",
            EnumToken::LeftParenthesis =>  "(",
            EnumToken::RightParenthesis => ")",
            EnumToken::Comma => ",",
            EnumToken::Attribute(_) => "attribute",
            EnumToken::NewLine => unreachable!(),
            EnumToken::Whitespace => unreachable!(),
        };

        write!(f, "{str}")
    }
}

impl TokenDefinition for EnumToken {
    fn keyword() -> &'static str {
        "enum"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }

    fn left_curly_bracket() -> Self { 
        Self::LeftCurlyBracket
    }

    fn right_curly_bracket() -> Self { 
        Self::RightCurlyBracket
    }
}

impl NamedStatement for EnumToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

pub(crate) fn enum_callback(
    lex: &mut Lexer<SchemaStatement>,
) -> Result<Vec<Token<EnumToken>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, EnumToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<EnumToken>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            EnumToken::NewLine => inner.extras.new_line(),
            EnumToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => {
                if let EnumToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let EnumToken::RightCurlyBracket = &kind {
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

