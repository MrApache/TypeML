use lexer_utils::{Position, Token, TokenType, KEYWORD, TYPE};
use logos::{Lexer, Logos};

use crate::SchemaTokens;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum GroupContext {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("[")]
    ArrayOpen,

    #[token("]")]
    ArrayClose,

    #[token("\n")]
    NewLine,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[regex(r"[ \t\r]+")]
    Whitespace,
}

impl TokenType for GroupContext {
    fn get_token_type(&self) -> u32 {
        match self {
            GroupContext::Keyword => KEYWORD,
            GroupContext::Identifier => TYPE,
            GroupContext::ArrayOpen => KEYWORD,
            GroupContext::ArrayClose => KEYWORD,
            GroupContext::NewLine => u32::MAX,
            GroupContext::Semicolon => u32::MAX,
            GroupContext::Comma => u32::MAX,
            GroupContext::Whitespace => u32::MAX,
        }
    }
}

pub(crate) fn group_context_callback(
    lex: &mut Lexer<SchemaTokens>,
) -> Option<Vec<Token<GroupContext>>> {

    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: GroupContext::Keyword,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip 'group'
    lex.extras.current_column += 5;

    let mut inner = lex.clone().morph::<GroupContext>();

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            GroupContext::Semicolon => {
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });

                inner.extras.current_column += inner.span().len() as u32;
                break;
            }
            GroupContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            GroupContext::Whitespace => {
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
