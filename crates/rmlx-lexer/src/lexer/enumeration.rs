use crate::{
    attribute_callback, AttributeToken, Error, NamedStatement, SchemaStatement, StatementTokens,
    TokenBodyStatement,
};
use lexer_utils::*;
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum EnumDefinitionToken {
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

    SyntaxError,
}

impl Display for EnumDefinitionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EnumDefinitionToken::Keyword => "enum",
            EnumDefinitionToken::Identifier => "identifier",
            EnumDefinitionToken::LeftCurlyBracket => "{",
            EnumDefinitionToken::RightCurlyBracket => "}",
            EnumDefinitionToken::LeftParenthesis => "(",
            EnumDefinitionToken::RightParenthesis => ")",
            EnumDefinitionToken::Comma => ",",
            EnumDefinitionToken::Attribute(_) => "attribute",
            EnumDefinitionToken::NewLine => unreachable!(),
            EnumDefinitionToken::Whitespace => unreachable!(),
            EnumDefinitionToken::SyntaxError => "error",
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for EnumDefinitionToken {
    fn keyword() -> &'static str {
        "enum"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl TokenBodyStatement for EnumDefinitionToken {
    fn left_curly_bracket() -> Self {
        Self::LeftCurlyBracket
    }

    fn right_curly_bracket() -> Self {
        Self::RightCurlyBracket
    }
}

impl NamedStatement for EnumDefinitionToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

pub(crate) fn enum_callback(lex: &mut Lexer<SchemaStatement>) -> Vec<Token<EnumDefinitionToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, EnumDefinitionToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<EnumDefinitionToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, EnumDefinitionToken::SyntaxError, &mut inner) {
            EnumDefinitionToken::NewLine => inner.extras.new_line(),
            EnumDefinitionToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => {
                if let EnumDefinitionToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                } else if let EnumDefinitionToken::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        Token::push_with_advance(&mut tokens, EnumDefinitionToken::SyntaxError, &mut inner);
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
