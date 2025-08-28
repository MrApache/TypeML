use logos::{Lexer, Logos};
use lexer_utils::*;
use crate::MarkupTokens;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum DirectiveToken {
    #[token("#")]
    Hash,

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
    NewLine,
}

pub(crate) fn directive_callback(
    lex: &mut Lexer<MarkupTokens>,
) -> Option<Vec<Token<DirectiveToken>>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, DirectiveToken::Hash, lex);

    let mut inner = lex.clone().morph::<DirectiveToken>();
    while let Some(token) = inner.next() {
        let kind = match token {
            Ok(kind) => kind,
            Err(_) => return None,
        };

        match kind {
            DirectiveToken::NewLine => new_line_and_break!(inner),
            DirectiveToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            _ => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    Some(tokens)
}
