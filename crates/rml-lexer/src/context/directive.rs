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

            DirectiveContext::Expression => 0,
            DirectiveContext::Import => 0,

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
        delta_start: lex.extras.get_delta_start(),
    });

    //Skip '#'
    lex.extras.current_column += 1;

    let mut inner = lex.clone().morph::<DirectiveContext>();
    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            DirectiveContext::End => {
                inner.extras.new_line();
                break;
            }
            DirectiveContext::Whitespace => {
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
