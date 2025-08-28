use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum ElementTokens {
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

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for ElementTokens {
    fn get_token_type(&self) -> u32 {
        match self {
            ElementTokens::Keyword => KEYWORD_TOKEN,
            ElementTokens::Identifier => TYPE_TOKEN,
            ElementTokens::LeftSquareBracket => KEYWORD_TOKEN,
            ElementTokens::RightSquareBracket => KEYWORD_TOKEN,
            ElementTokens::LeftCurlyBracket => u32::MAX,
            ElementTokens::RightCurlyBracket => u32::MAX,
            ElementTokens::LeftAngleBracket => OPERATOR_TOKEN,
            ElementTokens::RightAngleBracket => OPERATOR_TOKEN,
            ElementTokens::NewLine => u32::MAX,
            ElementTokens::Colon => u32::MAX,
            ElementTokens::Semicolon => u32::MAX,
            ElementTokens::Comma => u32::MAX,
            ElementTokens::Whitespace => u32::MAX,
        }
   }
}

pub(crate) fn element_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<ElementTokens>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, ElementTokens::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<ElementTokens>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            ElementTokens::NewLine => inner.extras.new_line(),
            ElementTokens::Semicolon => push_and_break!(&mut tokens, kind, &mut inner),
            ElementTokens::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => {
                if let ElementTokens::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let ElementTokens::RightCurlyBracket = &kind {
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
