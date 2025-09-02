use crate::{Error, SchemaStatement, StatementTokens};
use lexer_utils::{unwrap_or_continue, Position, Token};
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum DirectiveToken {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[regex(r#"<[^ \t\r\n><]*>"#)]
    Value,

    SyntaxError,
}

impl Display for DirectiveToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DirectiveToken::Keyword => "@",
            DirectiveToken::Identifier => "identifier",
            DirectiveToken::Value => "value",
            DirectiveToken::SyntaxError => "error",
            DirectiveToken::NewLine => "newline",
            DirectiveToken::Whitespace => "whitespace",
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for DirectiveToken {
    fn keyword() -> &'static str {
        "@"
    }

    fn keyword_token() -> Self {
        DirectiveToken::Keyword
    }
}

pub(crate) fn directive_callback(lex: &mut Lexer<SchemaStatement>) -> Vec<Token<DirectiveToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, DirectiveToken::Keyword, lex);

    let mut inner = lex.clone().morph::<DirectiveToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, DirectiveToken::SyntaxError, &mut inner) {
            DirectiveToken::NewLine => {
                inner.extras.new_line();
                break;
            }
            DirectiveToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    tokens
}
