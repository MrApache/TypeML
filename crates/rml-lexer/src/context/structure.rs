use lexer_utils::*;
use logos::{Lexer, Logos};

use crate::context::attribute::AttributeContext;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum StructToken {
    DoubleLeftCurlyBracket,

    #[token("}}")]
    DoubleRightCurlyBracket,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token(":")]
    Assing,

    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    #[regex(r"[0-9]+")]
    Int,

    #[token(",")]
    Comma,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for StructToken {
    fn get_token_type(&self) -> u32 {
        match self {
            StructToken::DoubleLeftCurlyBracket => 7,
            StructToken::DoubleRightCurlyBracket => 7,

            StructToken::Identifier => 1,
            StructToken::Float => 5,
            StructToken::Int => 5,

            StructToken::Assing => u32::MAX,
            StructToken::Comma => u32::MAX,
            StructToken::NewLine => u32::MAX,
            StructToken::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn struct_callback(
    lex: &mut Lexer<AttributeContext>,
) -> Option<Vec<Token<StructToken>>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, StructToken::DoubleLeftCurlyBracket, lex);

    let mut inner = lex.clone().morph::<StructToken>();
    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            StructToken::DoubleRightCurlyBracket => push_and_break!(&mut tokens, kind, &mut inner),
            StructToken::NewLine => inner.extras.new_line(),
            StructToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
