use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{attribute_callback, AttributeTokens, Error, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum EnumTokens {
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
    Attribute(Vec<Token<AttributeTokens>>),
}

impl TokenType for EnumTokens {
    fn get_token_type(&self) -> u32 {
        match self {
            EnumTokens::Keyword => KEYWORD,
            EnumTokens::Identifier => TYPE,
            EnumTokens::LeftCurlyBracket => u32::MAX,
            EnumTokens::RightCurlyBracket => u32::MAX,
            EnumTokens::LeftParenthesis => u32::MAX,
            EnumTokens::RightParenthesis => u32::MAX,
            EnumTokens::NewLine => u32::MAX,
            EnumTokens::Comma => u32::MAX,
            EnumTokens::Whitespace => u32::MAX,
            EnumTokens::Attribute(_) => unreachable!()
        }
    }
}

pub(crate) fn enum_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<EnumTokens>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, EnumTokens::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<EnumTokens>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            EnumTokens::NewLine => inner.extras.new_line(),
            EnumTokens::Whitespace => inner.extras.current_column += inner.span().len() as u32,
            _ => {
                if let EnumTokens::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let EnumTokens::RightCurlyBracket = &kind {
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

