use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum UseTokens {
    Keyword,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[regex(r#"<[^ \t\r\n><]*>"#)]
    Path,
}

impl TokenType for UseTokens {
    fn get_token_type(&self) -> u32 {
        match self {
            UseTokens::Keyword => KEYWORD,
            UseTokens::NewLine => u32::MAX,
            UseTokens::Whitespace => u32::MAX,
            UseTokens::Path => STRING,
        }
    }
}

pub(crate) fn use_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<UseTokens>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, UseTokens::Keyword, lex);

    let mut inner = lex.clone().morph::<UseTokens>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            UseTokens::NewLine => {
                inner.extras.new_line();
                break;
            },
            UseTokens::Whitespace => inner.extras.current_column += inner.span().len() as u32,
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Ok(tokens)
}
