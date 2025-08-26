use logos::{Lexer, Logos};
use lexer_utils::*;

use crate::{
    context::attribute::{attribute_context_callback, AttributeContext},
    MarkupTokens,
};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum TagContext {
    #[token("<")]
    TagStart,

    #[token("/")]
    Slash,

    #[token(">")]
    TagEnd,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("=", attribute_context_callback)]
    Attribute(Vec<Token<AttributeContext>>),

    TagIdentifier,

    AttributeIdentifier,
}

//SemanticTokenType::KEYWORD    -- 0
//SemanticTokenType::PARAMETER  -- 1
//SemanticTokenType::STRING     -- 2
//SemanticTokenType::TYPE       -- 3
//SemanticTokenType::OPERATOR   -- 4
//SemanticTokenType::VARIABLE   -- 5
impl TokenType for TagContext {
    fn get_token_type(&self) -> u32 {
        match self {
            TagContext::AttributeIdentifier => 1,

            TagContext::TagIdentifier => 3,

            TagContext::TagStart => 4,
            TagContext::Slash => 4,
            TagContext::TagEnd => 4,

            TagContext::NewLine => u32::MAX,
            TagContext::Whitespace => u32::MAX,

            TagContext::Identifier => unreachable!(),
            TagContext::Attribute(_) => 4,
        }
    }
}

pub(crate) fn tag_context_callback(
    lex: &mut Lexer<MarkupTokens>,
) -> Option<Vec<Token<TagContext>>> {
    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: TagContext::TagStart,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '<'
    lex.extras.current_column += 1;

    let mut inner = lex.clone().morph::<TagContext>();
    let mut tag_identifier = false;

    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };
        match kind {
            TagContext::TagEnd => {
                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: inner.extras.get_delta_start(),
                });

                inner.extras.current_column += inner.span().len() as u32;
                break;
            },
            TagContext::NewLine => {
                inner.extras.new_line();
                continue;
            }
            TagContext::Whitespace => {
                inner.extras.current_column += inner.span().len() as u32;
                continue;
            }
            _ => {
                let kind = if let TagContext::Identifier = kind {
                    if !tag_identifier {
                        tag_identifier = true;
                        TagContext::TagIdentifier
                    } else {
                        TagContext::AttributeIdentifier
                    }
                }
                else {
                    kind
                };

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
