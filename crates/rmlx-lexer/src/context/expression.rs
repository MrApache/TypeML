use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{Error, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum ExpressionTokens {
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

impl TokenType for ExpressionTokens {
    fn get_token_type(&self) -> u32 {
        match self {
            ExpressionTokens::Keyword => KEYWORD,
            ExpressionTokens::Identifier => TYPE,
            ExpressionTokens::LeftSquareBracket => KEYWORD,
            ExpressionTokens::RightSquareBracket => KEYWORD,
            ExpressionTokens::LeftCurlyBracket => u32::MAX,
            ExpressionTokens::RightCurlyBracket => u32::MAX,
            ExpressionTokens::LeftAngleBracket => OPERATOR,
            ExpressionTokens::RightAngleBracket => OPERATOR,
            ExpressionTokens::NewLine => u32::MAX,
            ExpressionTokens::Colon => u32::MAX,
            ExpressionTokens::Comma => u32::MAX,
            ExpressionTokens::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn expression_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Result<Vec<Token<ExpressionTokens>>, Error> {

    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, ExpressionTokens::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<ExpressionTokens>();
    while let Some(token) = inner.next() {
        let kind = token?;
        match kind {
            ExpressionTokens::NewLine => inner.extras.new_line(),
            ExpressionTokens::Whitespace => inner.extras.current_column += inner.span().len() as u32,
            _ => {
                if let ExpressionTokens::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let ExpressionTokens::RightCurlyBracket = &kind {
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

