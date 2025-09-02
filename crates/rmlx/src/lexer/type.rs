use crate::{
    Error, NamedStatement, SchemaStatement, StatementTokens, TokenBodyStatement,
    TokenSimpleTypeProvider,
};
use lexer_core::{push_and_break, unwrap_or_continue, Position, Token};
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum TypeDefinitionToken {
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

    #[token("-")]
    Dash,

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

    SyntaxError,
}

impl Display for TypeDefinitionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TypeDefinitionToken::Keyword => "keyword",
            TypeDefinitionToken::Identifier => "identifier",
            TypeDefinitionToken::LeftCurlyBracket => "{",
            TypeDefinitionToken::RightCurlyBracket => "}",
            TypeDefinitionToken::LeftAngleBracket => "<",
            TypeDefinitionToken::RightAngleBracket => ">",
            TypeDefinitionToken::Colon => ":",
            TypeDefinitionToken::Comma => ",",
            TypeDefinitionToken::Dash => "-",
            TypeDefinitionToken::Semicolon => ";",
            TypeDefinitionToken::NewLine => "newline",
            TypeDefinitionToken::Whitespace => "whitespace",
            TypeDefinitionToken::SyntaxError => "syntax error",
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for TypeDefinitionToken {
    fn keyword() -> &'static str {
        "element"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl TokenBodyStatement for TypeDefinitionToken {
    fn left_curly_bracket() -> Self {
        Self::LeftCurlyBracket
    }

    fn right_curly_bracket() -> Self {
        Self::RightCurlyBracket
    }
}

impl NamedStatement for TypeDefinitionToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

impl TokenSimpleTypeProvider for TypeDefinitionToken {
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

pub(crate) fn type_callback(lex: &mut Lexer<SchemaStatement>) -> Vec<Token<TypeDefinitionToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, TypeDefinitionToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<TypeDefinitionToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(
            token,
            &mut tokens,
            TypeDefinitionToken::SyntaxError,
            &mut inner
        ) {
            TypeDefinitionToken::NewLine => inner.extras.new_line(),
            TypeDefinitionToken::Semicolon => {
                push_and_break!(&mut tokens, TypeDefinitionToken::Semicolon, &mut inner)
            }
            TypeDefinitionToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => {
                if let TypeDefinitionToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                } else if let TypeDefinitionToken::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        Token::push_with_advance(
                            &mut tokens,
                            TypeDefinitionToken::SyntaxError,
                            &mut inner,
                        );
                        return tokens;
                    }
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        push_and_break!(&mut tokens, kind, &mut inner);
                    }
                }
                Token::push_with_advance(&mut tokens, kind, &mut inner);
            }
        }
    }

    *lex = inner.morph();
    tokens
}
