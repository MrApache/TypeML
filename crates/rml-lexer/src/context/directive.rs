use logos::{Lexer, Logos};

use crate::{DefaultContext, Position, Token, TokenType};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum DirectiveContext {
    #[token("#")]
    Start,

    #[token("expressions", priority = 1)]
    Expression,

    #[token("import", priority = 1)]
    Import,

    #[token("as", priority = 1)]
    As,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", priority = 0)]
    Alias,

    #[regex(r#""[a-zA-Z0-9_\-./]+""#, priority = 0)]
    Path,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    #[token("\n")]
    End,
}

//SemanticTokenType::KEYWORD   -- 0
//SemanticTokenType::VARIABLE  -- 1
//SemanticTokenType::STRING    -- 2
impl TokenType for DirectiveContext {
    fn get_token_type(&self) -> u32 {
        match self {
            DirectiveContext::Start => 0,
            DirectiveContext::As => 0,

            DirectiveContext::Whitespace => 0,
            DirectiveContext::End => 0,

            DirectiveContext::Expression => 1,
            DirectiveContext::Import => 1,

            DirectiveContext::Alias => 1,

            DirectiveContext::Path => 2,
        }
    }
}

pub(crate) fn directive_context_callback(
    lex: &mut Lexer<DefaultContext>,
) -> Option<Vec<Token<DirectiveContext>>> {
    let mut tokens = Vec::new();
    tokens.push(Token {
        kind: DirectiveContext::Start,
        span: lex.span(),
        delta_line: lex.extras.get_delta_line(),
        delta_start: 0,
        length: lex.span().len() as u32,
    });

    let mut inner = lex.clone().morph::<DirectiveContext>();
    let mut delta_start = 1;

    while let Some(token) = inner.next() {
        match token {
            Ok(kind) => {
                if kind == DirectiveContext::End {
                    inner.extras.current_line += 1;
                    inner.extras.previous_token_end_column = 0;
                    inner.extras.current_column = 0;
                    break;
                }
                if kind == DirectiveContext::Whitespace {
                    delta_start += inner.span().len();
                    continue;
                }

                tokens.push(Token {
                    kind,
                    span: inner.span(),
                    delta_line: inner.extras.get_delta_line(),
                    delta_start: delta_start as u32 - inner.extras.previous_token_end_column,
                    length: inner.span().len() as u32,
                });
                inner.extras.previous_token_end_column = delta_start as u32;
                delta_start += inner.span().len();
                inner.extras.current_line = delta_start as u32;
            }
            Err(_) => return None,
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
