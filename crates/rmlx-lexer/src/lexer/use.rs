use crate::{Error, SchemaStatement, StatementTokens};
use lexer_utils::*;
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum UseToken {
    Keyword,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[regex(r#"<[^ \t\r\n><]*>"#)]
    Path,

    SyntaxError,
}

impl Display for UseToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UseToken::Keyword => "use",
            UseToken::Path => "path",
            UseToken::SyntaxError => "error",
            UseToken::NewLine => unreachable!(),
            UseToken::Whitespace => unreachable!(),
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for UseToken {
    fn keyword() -> &'static str {
        "use"
    }

    fn keyword_token() -> Self {
        UseToken::Keyword
    }
}

pub(crate) fn use_callback(lex: &mut Lexer<SchemaStatement>) -> Vec<Token<UseToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, UseToken::Keyword, lex);

    let mut inner = lex.clone().morph::<UseToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, UseToken::SyntaxError, &mut inner) {
            UseToken::NewLine => {
                inner.extras.new_line();
                break;
            }
            UseToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    tokens
}
