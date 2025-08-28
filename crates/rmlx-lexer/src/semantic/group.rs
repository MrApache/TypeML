use std::slice::Iter;

use lexer_utils::{Token, KEYWORD_TOKEN, TYPE_TOKEN};
use tower_lsp::lsp_types::SemanticToken;

use crate::{semantic::Attribute, GroupToken};

#[derive(Debug)]
pub struct Group {
    name: String,
    groups: Vec<String>,
    min: Option<u32>,
    max: Option<u32>,
    extend: bool,
}

impl Group {
    fn new(name: String, groups: Vec<String>) -> Self {
        Self {
            name,
            groups,
            min: None,
            max: None,
            extend: false,
        }
    }

    pub fn resolve_attributes(&mut self, attributes: &mut Vec<Attribute>) {
        attributes.retain(|attr| {
            match attr {
                Attribute::Extend => self.extend = true,
                Attribute::Min(min_attribute) => self.min = Some(min_attribute.value),
                Attribute::Max(max_attribute) => self.max = Some(max_attribute.value),
                _ => return false,
            }

            true
        });
    }
}

pub fn parse_group<'s>(
    mut iter: Iter<'s, Token<GroupToken>>,
    src: &str,
    tokens: &mut Vec<SemanticToken>,
) -> Result<Group, String> {
    // 1. читаем `group`
    let t = iter.next().ok_or("Expected `group` keyword")?;
    if t.kind() != &GroupToken::Keyword {
        return Err(format!("Expected `group`, got {:?}", t.kind()));
    }
    tokens.push(t.to_semantic_token(KEYWORD_TOKEN));

    // 2. читаем идентификатор (имя группы)
    let t = iter.next().ok_or("Expected identifier after `group`")?;
    if t.kind() != &GroupToken::Identifier {
        return Err(format!("Expected identifier, got {:?}", t.kind()));
    }
    let name = t.slice(src).to_string();
    tokens.push(t.to_semantic_token(TYPE_TOKEN));

    let mut groups = Vec::new();

    // 3. читаем следующий токен
    let t = iter.next().ok_or("Expected `;` or `[` after identifier")?;

    match t.kind() {
        GroupToken::Semicolon => {
            tokens.push(t.to_semantic_token(u32::MAX));
            Ok(Group::new(name, groups))
        }
        GroupToken::LeftSquareBracket => {
            tokens.push(t.to_semantic_token(u32::MAX));

            loop {
                // читаем идентификатор
                let t = iter.next().ok_or("Expected identifier inside `[]`")?;
                if t.kind() != &GroupToken::Identifier {
                    return Err(format!("Expected identifier inside `[]`, got {:?}", t.kind()));
                }
                tokens.push(t.to_semantic_token(TYPE_TOKEN));
                groups.push(t.slice(src).to_string());

                // читаем либо `,` либо `]`
                let t = iter.next().ok_or("Expected `,` or `]` after identifier")?;
                match t.kind() {
                    GroupToken::Comma => {
                        tokens.push(t.to_semantic_token(u32::MAX));
                        continue;
                    }
                    GroupToken::RightSquareBracket => {
                        tokens.push(t.to_semantic_token(u32::MAX));
                        break;
                    }
                    _ => return Err(format!("Expected `,` or `]`, got {:?}", t.kind())),
                }
            }

            // после `]` обязательно `;`
            let t = iter.next().ok_or("Expected `;` after group declaration")?;
            if t.kind() != &GroupToken::Semicolon {
                return Err(format!("Expected `;` after group declaration, got {:?}", t.kind()));
            }
            tokens.push(t.to_semantic_token(u32::MAX));

            Ok(Group::new(name, groups))
        }
        _ => Err(format!("Expected `;` or `[`, got {:?}", t.kind())),
    }
}

#[cfg(test)]
mod tests {
    use crate::{semantic::parse_group, RmlxTokenStream, SchemaTokens};

    #[test]
    fn test() {
        const CONTENT: &str = r#"group Container;"#;

        let tokens = RmlxTokenStream::new(CONTENT).to_vec().unwrap();
        let tokens = tokens.first().unwrap();
        if let SchemaTokens::Group(tokens) = tokens {
            let xd = parse_group(tokens.iter(), CONTENT, &mut vec![]);
            println!();
        }
    }
}
