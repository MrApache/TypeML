use crate::context::attribute::AttributeContext;
use lexer_core::*;
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum ExpressionToken {
    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token(":")]
    Colon,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

pub(crate) fn expression_callback(
    lex: &mut logos::Lexer<AttributeContext>,
) -> Option<Vec<Token<ExpressionToken>>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, ExpressionToken::LeftCurlyBracket, lex);

    let mut inner = lex.clone().morph::<ExpressionToken>();
    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            ExpressionToken::RightCurlyBracket => push_and_break!(&mut tokens, kind, &mut inner),
            ExpressionToken::NewLine => inner.extras.new_line(),
            ExpressionToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
