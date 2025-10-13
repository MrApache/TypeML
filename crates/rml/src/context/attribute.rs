use crate::context::{
    expression::ExpressionToken, expression_callback, struct_callback,
    structure::StructToken, tag::TagContext,
};
use lexer_core::{push_and_break, Position, Token};
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum AttributeContext {
    #[token("\"")]
    Quote,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Value,

    #[regex(r"[0-9]+\.[0-9]+")]
    Float,

    #[regex(r"[0-9]+")]
    Int,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    Equal,

    #[token("{", expression_callback)]
    Expression(Vec<Token<ExpressionToken>>),

    #[token("{{", struct_callback)]
    Struct(Vec<Token<StructToken>>),
}

pub(crate) fn attribute_context_callback(
    lex: &mut Lexer<TagContext>,
) -> Option<Vec<Token<AttributeContext>>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, AttributeContext::Equal, lex);

    let mut inner = lex.clone().morph::<AttributeContext>();
    //Read first quote
    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            AttributeContext::Quote => push_and_break!(&mut tokens, kind, &mut inner),
            AttributeContext::NewLine => inner.extras.new_line(),
            AttributeContext::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => panic!(),
        }
    }

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            //Read last quote and break
            AttributeContext::Quote => push_and_break!(&mut tokens, kind, &mut inner),
            AttributeContext::NewLine => inner.extras.new_line(),
            AttributeContext::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
