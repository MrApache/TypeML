use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::SchemaTokens;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum ExpressionContext {
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

impl TokenType for ExpressionContext {
    fn get_token_type(&self) -> u32 {
        match self {
            ExpressionContext::Keyword => KEYWORD,
            ExpressionContext::Identifier => TYPE,
            ExpressionContext::LeftSquareBracket => KEYWORD,
            ExpressionContext::RightSquareBracket => KEYWORD,
            ExpressionContext::LeftCurlyBracket => u32::MAX,
            ExpressionContext::RightCurlyBracket => u32::MAX,
            ExpressionContext::LeftAngleBracket => OPERATOR,
            ExpressionContext::RightAngleBracket => OPERATOR,
            ExpressionContext::NewLine => u32::MAX,
            ExpressionContext::Colon => u32::MAX,
            ExpressionContext::Comma => u32::MAX,
            ExpressionContext::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn expression_context_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Option<Vec<Token<ExpressionContext>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: ExpressionContext::Keyword,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip 'expression'
    lex.extras.current_column += 10;

    let mut inner = lex.clone().morph::<ExpressionContext>();

    let mut bracket_depth = 0;

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            ExpressionContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            ExpressionContext::Whitespace => {
                inner.extras.current_column += inner.span().len() as u32;
                continue;
            }
            _ => {
                if let ExpressionContext::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                }
                else if let ExpressionContext::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        panic!();
                    }
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        tokens.push(Token {
                            kind,
                            span: inner.span(),
                            delta_line: inner.extras.get_delta_line(),
                            delta_start: inner.extras.get_delta_start(),
                        });

                        inner.extras.current_column += inner.span().len() as u32;
                        break;
                    }
                }
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

