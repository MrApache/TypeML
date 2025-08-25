use logos::{Lexer, Logos};

use crate::{
    context::{
        expression::ExpressionContext, expression_context_callback, struct_context_callback,
        structure::StructContext, tag::TagContext,
    },
    Position, Token, TokenType,
};

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

    #[token("{", expression_context_callback)]
    Expression(Vec<Token<ExpressionContext>>),

    #[token("{{", struct_context_callback)]
    Struct(Vec<Token<StructContext>>),
}

impl TokenType for AttributeContext {
    fn get_token_type(&self) -> u32 {
        match self {
            AttributeContext::Quote => 2,
            AttributeContext::Equal => 4,
            AttributeContext::Value => 5,
            AttributeContext::Float => 5,
            AttributeContext::Int => 5,
            AttributeContext::NewLine => u32::MAX,
            AttributeContext::Whitespace => u32::MAX,
            AttributeContext::Expression(_) => unreachable!(),
            AttributeContext::Struct(_) => unreachable!(),
        }
    }
}

pub(crate) fn attribute_context_callback(
    lex: &mut Lexer<TagContext>,
) -> Option<Vec<Token<AttributeContext>>> {
    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: AttributeContext::Equal,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '='
    lex.extras.current_column += 1;

    let mut inner = lex.clone().morph::<AttributeContext>();

    //Read first quote
    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            AttributeContext::Quote => {
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });
                inner.extras.current_column += inner.span().len() as u32;
                break;
            }
            AttributeContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            AttributeContext::Whitespace => {
                inner.extras.current_column += inner.span().len() as u32;
                continue;
            }
            _ => panic!(),
        }
    }

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            AttributeContext::Quote => {
                //Read last quote and break
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });
                inner.extras.current_column += inner.span().len() as u32;
                break;
            }
            AttributeContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            AttributeContext::Whitespace => {
                inner.extras.current_column += inner.span().len() as u32;
                continue;
            }
            _ => {
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });
                inner.extras.current_column += inner.span().len() as u32;
            }
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
