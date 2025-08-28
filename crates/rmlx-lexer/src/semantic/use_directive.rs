use std::slice::Iter;

use lexer_utils::{Token, KEYWORD_TOKEN, STRING_TOKEN};
use tower_lsp::lsp_types::SemanticToken;

use crate::UseToken;

pub struct Use {
    path: String,
}

pub fn parse_use<'s>(
    mut iter: Iter<'s, Token<UseToken>>,
    src: &str,
    tokens: &mut Vec<SemanticToken>,
) -> Result<Use, String> {

    let t = iter.next().ok_or("Expected `use` keyword")?;
    if t.kind() != &UseToken::Keyword {
        return Err(format!("Expected `use`, got {:?}", t.kind()));
    }
    tokens.push(t.to_semantic_token(KEYWORD_TOKEN));

    let t = iter.next().ok_or("Expected <...> path")?;
    if t.kind() != &UseToken::Path {
        return Err(format!("Expected path, got {:?}", t.kind()));
    }
    let path = src
        .get(t.span())
        .map(trim_angle_brackets)
        .ok_or("Invalid span for path")?
        .to_string();

    tokens.push(t.to_semantic_token(STRING_TOKEN));

    Ok(Use { path })
}

fn trim_angle_brackets(path: &str) -> &str {
    path.strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use crate::{semantic::parse_use, RmlxTokenStream, SchemaTokens};

    #[test]
    fn test() {
        const CONTENT: &str = r#"use <../base.rmlx>"#;

        let tokens = RmlxTokenStream::new(CONTENT).to_vec().unwrap();
        let tokens = tokens.first().unwrap();
        if let SchemaTokens::Use(tokens) = tokens {
            let xd = parse_use(tokens.iter(), CONTENT, &mut vec![]);
            println!();
        }
    }
}
