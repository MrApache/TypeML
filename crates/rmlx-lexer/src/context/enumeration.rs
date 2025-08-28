use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{attribute_callback, AttributeToken, Error, SchemaTokens};

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

impl TokenType for EnumToken {
    fn get_token_type(&self) -> u32 {
        match self {
            EnumToken::Keyword => KEYWORD_TOKEN,
            EnumToken::Identifier => TYPE_TOKEN,
            EnumToken::LeftCurlyBracket => u32::MAX,
            EnumToken::RightCurlyBracket => u32::MAX,
            EnumToken::LeftParenthesis => u32::MAX,
            EnumToken::RightParenthesis => u32::MAX,
            EnumToken::NewLine => u32::MAX,
            EnumToken::Comma => u32::MAX,
            EnumToken::Whitespace => u32::MAX,
            EnumToken::Attribute(_) => unreachable!()
        }
    }
}

pub(crate) fn enum_callback(
    lex: &mut Lexer<SchemaTokens>,
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

