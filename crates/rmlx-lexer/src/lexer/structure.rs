use crate::{
    Error, NamedStatement, SchemaStatement, StatementTokens, TokenBodyStatement,
    TokenSimpleTypeProvider,
};
use lexer_utils::*;
use logos::{Lexer, Logos};
use std::fmt::Display;

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

    SyntaxError,
}

impl StatementTokens for StructToken {
    fn keyword() -> &'static str {
        "struct"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl TokenBodyStatement for StructToken {
    fn left_curly_bracket() -> Self {
        Self::LeftCurlyBracket
    }

    fn right_curly_bracket() -> Self {
        Self::RightCurlyBracket
    }
}

impl TokenSimpleTypeProvider for StructToken {
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

impl NamedStatement for StructToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

impl Display for StructToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StructToken::Keyword => "struct",
            StructToken::Identifier => "identifier",
            StructToken::SyntaxError => "error",
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

pub(crate) fn struct_callback(
    lex: &mut Lexer<SchemaStatement>,
) -> Vec<Token<StructToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, StructToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<StructToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, StructToken::SyntaxError, &mut inner) {
            StructToken::NewLine => inner.extras.new_line(),
            StructToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => {
                if let StructToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                } else if let StructToken::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        Token::push_with_advance(&mut tokens, StructToken::SyntaxError, &mut inner);
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
