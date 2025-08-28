use std::slice::Iter;
use tower_lsp::lsp_types::SemanticToken;
use lexer_utils::{Token, MACRO_TOKEN, STRING_TOKEN};
use crate::{AttributeToken, ContentToken};

pub enum Attribute {
    Extend,
    Pattern(PatternAttribute),
    Group(GroupAttribute),
    Path(PathAttribute),
    Min(MinAttribute),
    Max(MaxAttribute),
}

pub struct PathAttribute {
    pub value: String,
}

pub struct GroupAttribute {
    pub value: String,
}

pub struct MinAttribute {
    pub value: u32,
}

pub struct MaxAttribute {
    pub value: u32,
}

pub struct PatternAttribute {
    pub value: String,
}

pub fn parse_attributes<'s>(
    mut iter: Iter<'s, Token<AttributeToken>>,
    src: &str,
    tokens: &mut Vec<SemanticToken>,
) -> Result<Vec<Attribute>, String> {
    let mut attrs = Vec::new();

    {
        // читаем `#`
        let t = iter.next().ok_or("Expected #")?;
        tokens.push(t.to_semantic_token(MACRO_TOKEN));
        if t.kind() != &AttributeToken::Hash {
            return Err(format!("Expected #, got {:?}", t.kind()));
        }
    }

    {
        // читаем `[`
        let t = iter.next().ok_or("Expected [")?;
        tokens.push(t.to_semantic_token(MACRO_TOKEN));
        if t.kind() != &AttributeToken::LeftSquareBracket {
            return Err(format!("Expected [, got {:?}", t.kind()));
        }
    }

    loop {
        // читаем идентификатор
        let t = iter.next().ok_or("Expected identifier")?;
        tokens.push(t.to_semantic_token(MACRO_TOKEN));
        if t.kind() != &AttributeToken::Identifier {
            return Err(format!("Expected identifier, got {:?}", t.kind()));
        }

        let name = src
            .get(t.span().clone())
            .ok_or("Invalid span for identifier")?
            .to_string();

        // читаем контент (может отсутствовать)
        let next = iter.next().ok_or("Expected content or , or ]")?;
        match next.kind() {
            AttributeToken::Content(inner_tokens) => {
                let value_str = parse_content(inner_tokens.iter(), src, tokens)?;

                let attr = match name.as_str() {
                    "Path" => Attribute::Path(PathAttribute { value: value_str }),
                    "Group" => Attribute::Group(GroupAttribute { value: value_str }),
                    "Min" => Attribute::Min(MinAttribute {
                        value: parse_min_value(&value_str)?,
                    }),
                    "Max" => Attribute::Max(MaxAttribute {
                        value: parse_max_value(&value_str)?,
                    }),
                    "Pattern" => Attribute::Pattern(PatternAttribute { value: value_str }),
                    "Extend" => return Err("Attribute 'Extend' does not have a content".into()),
                    _ => return Err(format!("Unknown attribute `{name}`")),
                };

                attrs.push(attr);
            }
            AttributeToken::RightSquareBracket => {
                tokens.push(next.to_semantic_token(u32::MAX));
                match name.as_str() {
                    "Extend" => attrs.push(Attribute::Extend),
                    _ => return Err(format!("Attribute `{name}` requires value")),
                }
                break;
            }
            AttributeToken::Comma => {
                tokens.push(next.to_semantic_token(u32::MAX));
                match name.as_str() {
                    "Extend" => attrs.push(Attribute::Extend),
                    _ => return Err(format!("Attribute `{name}` requires value")),
                }
                continue;
            }
            _ => {
                tokens.push(next.to_semantic_token(u32::MAX));
                return Err(format!(
                    "Unexpected token after identifier: {:?}",
                    next.kind()
                ))
            }
        }

        let next = iter.next().ok_or("Expected , or ]")?;
        if next.kind() == &AttributeToken::Comma {
            tokens.push(next.to_semantic_token(u32::MAX));
            continue;
        }

        if next.kind() == &AttributeToken::RightSquareBracket {
            tokens.push(next.to_semantic_token(MACRO_TOKEN));
            break;
        }
    }

    Ok(attrs)
}

fn parse_min_value(value_str: &str) -> Result<u32, String> {
    Ok(value_str.parse::<u32>().map_err(|_| "Invalid Min value")?)
}

fn parse_max_value(value_str: &str) -> Result<u32, String> {
    Ok(value_str.parse::<u32>().map_err(|_| "Invalid Max value")?)
}

pub fn parse_content<'s>(
    mut iter: Iter<'s, Token<ContentToken>>,
    src: &str,
    tokens: &mut Vec<SemanticToken>,
) -> Result<String, String> {
    let t = iter.next().ok_or("Expected '('")?;
    if t.kind() != &ContentToken::LeftParenthesis {
        return Err(format!("Expected '(', got {:?}", t.kind()));
    }
    tokens.push(t.to_semantic_token(u32::MAX));

    let t = iter.next().ok_or("Expected String or Value")?;
    let result_text = t.slice(src);
    let (semantic_token, result_text) = match t.kind() {
        ContentToken::String => {
            let text = t.slice(src).trim_matches('"');
            (t.to_semantic_token(STRING_TOKEN), text)
        },
        ContentToken::Value => (t.to_semantic_token(STRING_TOKEN), result_text),
        _ => return Err(format!("Expected String or Value, got {:?}", t.kind())),
    };
    tokens.push(semantic_token);

    // Читаем ')'
    let t = iter.next().ok_or("Expected ')'")?;
    if t.kind() != &ContentToken::RightParenthesis {
        return Err(format!("Expected ')', got {:?}", t.kind()));
    }
    tokens.push(t.to_semantic_token(u32::MAX));

    Ok(result_text.to_string())
}

#[cfg(test)]
mod tests {
    use crate::semantic::attribute::parse_attributes;
    use crate::{RmlxTokenStream, SchemaStatement};

    #[test]
    fn test() {
        const CONTENT: &str = r#"#[Path(std::iter), Min(0), Max(1), Pattern("Hello, world!")]"#;

        let tokens = RmlxTokenStream::new(CONTENT).to_vec().unwrap();
        let attr_tokens = tokens.first().unwrap();
        if let SchemaStatement::Attribute(tokens) = attr_tokens.clone() {
            let xd = parse_attributes(tokens.iter(), CONTENT, &mut vec![]);
            println!();
        }
    }
}
