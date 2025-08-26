use lexer_utils::*;
use logos::{Lexer, Logos};
use crate::{EnumContext, SchemaTokens};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum AttributeContext {
    Hash,

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    CloseSquareBracket,

    #[token(",")]
    Comma,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("(", content_callback)]
    Content(Vec<Token<Content>>),
}

impl TokenType for AttributeContext {
    fn get_token_type(&self) -> u32 {
        match self {
            AttributeContext::Hash => MACRO,
            AttributeContext::OpenSquareBracket => MACRO,
            AttributeContext::CloseSquareBracket => MACRO,
            AttributeContext::Comma => u32::MAX,
            AttributeContext::Identifier => MACRO,
            AttributeContext::NewLine => MACRO,
            AttributeContext::Whitespace => MACRO,
            AttributeContext::Content(_) => unreachable!(), 
        }
    }
}

pub(crate) fn attribute_context_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Option<Vec<Token<AttributeContext>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: AttributeContext::Hash,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '#'
    lex.extras.current_column += 1;

    let mut inner = lex.clone().morph::<AttributeContext>();

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            AttributeContext::CloseSquareBracket => {
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


pub(crate) fn attribute_context_callback_enum(
    lex: &mut Lexer<EnumContext>
) -> Option<Vec<Token<AttributeContext>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: AttributeContext::Hash,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '#'
    lex.extras.current_column += 1;

    let mut inner = lex.clone().morph::<AttributeContext>();

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            AttributeContext::CloseSquareBracket => {
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


#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum Content {
    OpenParenthesis,

    #[regex(r"[^\n)]+", priority = 0)]
    Value,

    #[regex(r#""[^\n"]+""#, priority = 1)]
    String,

    #[token(")")]
    CloseParenthesis,

    #[token("\n")]
    NewLine,
}

impl TokenType for Content {
    fn get_token_type(&self) -> u32 {
        match self {
            Content::Value => u32::MAX,
            Content::String => STRING,
            _ => MACRO
        }
    }
}

fn content_callback(
    lex: &mut Lexer<AttributeContext>,
) -> Option<Vec<Token<Content>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: Content::OpenParenthesis,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '('
    lex.extras.current_column += 1;

    let mut inner = lex.clone().morph::<Content>();

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            Content::CloseParenthesis => {
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });

                inner.extras.current_column += inner.span().len() as u32;
                break;
            },
            Content::NewLine => {
                inner.extras.new_line();
                continue;
            },
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
