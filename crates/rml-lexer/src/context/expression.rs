use logos::Logos;
use lexer_utils::*;
use crate::context::attribute::AttributeContext;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum ExpressionContext {
    #[token("{")]
    Start,

    #[token("}")]
    End,

    #[token(":")]
    Colon,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for ExpressionContext {
    fn get_token_type(&self) -> u32 {
        match self {
            ExpressionContext::Start => 4,
            ExpressionContext::End => 4,
            ExpressionContext::Identifier => 8,
            ExpressionContext::Colon => u32::MAX,
            ExpressionContext::NewLine => u32::MAX,
            ExpressionContext::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn expression_context_callback(lex: &mut logos::Lexer<AttributeContext>) -> Option<Vec<Token<ExpressionContext>>> {
    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: ExpressionContext::Start,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '{'
    lex.extras.current_column += 1;

    let mut inner = lex.clone().morph::<ExpressionContext>();
    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                match kind {
                    ExpressionContext::End => {
                        tokens.push(Token {
                            kind,
                            span: inner.span(),
                            delta_line: inner.extras.get_delta_line(),
                            delta_start: inner.extras.get_delta_start(),
                        });
                        inner.extras.current_column += inner.span().len() as u32;
                        break;
                    },
                    ExpressionContext::NewLine => {
                        inner.extras.new_line();
                        continue;
                    },
                    ExpressionContext::Whitespace => {
                        inner.extras.current_column += inner.span().len() as u32;
                        continue;
                    },
                    _ => {},
                }

                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });
                inner.extras.current_column += inner.span().len() as u32;
            }
            Err(_) => return None,
        }
    }

    *lex = inner.morph();
    Some(tokens)
}

