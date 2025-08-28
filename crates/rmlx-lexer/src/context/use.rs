use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, SchemaTokens};

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
}

impl TokenType for UseToken {
    fn get_token_type(&self) -> u32 {
        match self {
            UseToken::Keyword => KEYWORD_TOKEN,
            UseToken::NewLine => u32::MAX,
            UseToken::Whitespace => u32::MAX,
            UseToken::Path => STRING_TOKEN,
        }
    }
}

pub(crate) fn use_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<UseToken>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, UseToken::Keyword, lex);

    let mut inner = lex.clone().morph::<UseToken>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            UseToken::NewLine => {
                inner.extras.new_line();
                break;
            },
            UseToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}
