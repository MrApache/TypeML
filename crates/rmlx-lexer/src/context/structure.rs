use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum StructTokens {
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

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for StructTokens {
    fn get_token_type(&self) -> u32 {
        match self {
            StructTokens::Keyword => KEYWORD,
            StructTokens::Identifier => TYPE,
            StructTokens::LeftSquareBracket => KEYWORD,
            StructTokens::RightSquareBracket => KEYWORD,
            StructTokens::LeftCurlyBracket => u32::MAX,
            StructTokens::RightCurlyBracket => u32::MAX,
            StructTokens::LeftAngleBracket => OPERATOR,
            StructTokens::RightAngleBracket => OPERATOR,
            StructTokens::NewLine => u32::MAX,
            StructTokens::Colon => u32::MAX,
            StructTokens::Comma => u32::MAX,
            StructTokens::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn struct_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<StructTokens>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, StructTokens::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<StructTokens>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            StructTokens::NewLine => inner.extras.new_line(),
            StructTokens::Whitespace => inner.extras.current_column += inner.span().len() as u32,
            _ => {
                if let StructTokens::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let StructTokens::RightCurlyBracket = &kind {
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

