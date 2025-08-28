use lexer_utils::{Token, KEYWORD_TOKEN, PARAMETER_TOKEN, TYPE_TOKEN};
use tower_lsp::lsp_types::SemanticToken;

use crate::{
    semantic::{parse_attributes, Attribute},
    EnumToken,
};
use std::slice::Iter;

/// Всё перечисление
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

/// Один вариант в enum
pub struct EnumVariant {
    pub name: String,
    pub value_type: Option<String>, // None = простой вариант, Some("f32") = вариант с аргументом
    pub attributes: Vec<Attribute>, // Если к варианту были применены атрибуты #[...]
}

pub fn parse_enum<'s>(
    mut iter: Iter<'s, Token<EnumToken>>,
    src: &'s str,
    sm_tokens: &mut Vec<SemanticToken>,
) -> Result<Enum, String> {
    // 1. читаем `enum`
    let t = iter.next().ok_or("Expected `enum` keyword")?;
    if t.kind() != &EnumToken::Keyword {
        return Err(format!("Expected `enum`, got {:?}", t.kind()));
    }
    sm_tokens.push(t.to_semantic_token(KEYWORD_TOKEN));

    // 2. читаем имя enum
    let t = iter.next().ok_or("Expected enum name identifier")?;
    if t.kind() != &EnumToken::Identifier {
        return Err(format!(
            "Expected identifier for enum name, got {:?}",
            t.kind()
        ));
    }
    let enum_name = t.slice(src).to_string();
    sm_tokens.push(t.to_semantic_token(TYPE_TOKEN));

    // 3. читаем '{'
    let t = iter.next().ok_or("Expected `{` after enum name")?;
    if t.kind() != &EnumToken::LeftCurlyBracket {
        return Err(format!("Expected '{{', got {:?}", t.kind()));
    }
    sm_tokens.push(t.to_semantic_token(u32::MAX));

    let mut variants = Vec::new();
    let mut attributes = Vec::new();
    // 4. читаем варианты
    loop {
        let token = iter
            .next()
            .ok_or_else(|| "Unexpected end of tokens while parsing enum".to_string())?;

        match token.kind() {
            // конец enum
            EnumToken::RightCurlyBracket => {
                iter.next(); // съесть '}'
                sm_tokens.push(token.to_semantic_token(u32::MAX));
                break;
            }

            // атрибуты перед вариантом (0 или более)
            EnumToken::Attribute(tokens) => {
                attributes = parse_attributes(tokens.iter(), src, sm_tokens)?;
            }

            // имя варианта
            EnumToken::Identifier => {
                let name = token.slice(src).to_string();
                sm_tokens.push(token.to_semantic_token(PARAMETER_TOKEN));

                // проверяем, есть ли '(' для аргумента
                let value_type = if let Some(t) = iter.clone().next() {
                    if t.kind() == &EnumToken::LeftParenthesis {
                        iter.next(); // съесть '('
                        sm_tokens.push(t.to_semantic_token(u32::MAX));

                        // читаем тип значения
                        let t = iter.next().ok_or("Expected type inside parentheses")?;
                        if t.kind() != &EnumToken::Identifier {
                            return Err(format!(
                                "Expected type identifier inside '()', got {:?}",
                                t.kind()
                            ));
                        }
                        let typ = t.slice(src).to_string();
                        sm_tokens.push(t.to_semantic_token(TYPE_TOKEN));

                        // читаем ')'
                        let t = iter.next().ok_or("Expected ')' after type")?;
                        if t.kind() != &EnumToken::RightParenthesis {
                            return Err(format!("Expected ')', got {:?}", t.kind()));
                        }
                        sm_tokens.push(t.to_semantic_token(u32::MAX));

                        Some(typ)
                    } else {
                        None
                    }
                } else {
                    None
                };

                variants.push(EnumVariant {
                    name,
                    value_type,
                    attributes: std::mem::take(&mut attributes),
                });

                // после варианта может быть ',' или '}'
                if let Some(t) = iter.clone().next() {
                    match t.kind() {
                        EnumToken::Comma => {
                            iter.next(); // съесть ','
                            sm_tokens.push(t.to_semantic_token(u32::MAX));
                        }
                        EnumToken::RightCurlyBracket => {} // конец enum, обработаем в начале цикла
                        _ => return Err(format!("Expected ',' or '}}', got {:?}", t.kind())),
                    }
                }
            }
            _ => return Err(format!("Expected variant, got {:?}", token.kind())),
        }
    }

    Ok(Enum {
        name: enum_name,
        variants,
    })
}

#[cfg(test)]
mod tests {
    use crate::{semantic::parse_enum, RmlxTokenStream, SchemaTokens};

    #[test]
    fn test() {
        const CONTENT: &str = r#"
        enum Val {
            Auto,
        
            VMin(f32),
        
            #[Pattern("([0-9]+(?:\.[0-9]+)?)vmax")]
            VMax(f32),
        }
        "#;

        let tokens = RmlxTokenStream::new(CONTENT).to_vec().unwrap();
        let tokens = tokens.first().unwrap();
        if let SchemaTokens::Enum(tokens) = tokens {
            let xd = parse_enum(tokens.iter(), CONTENT, &mut vec![]);
            println!();
        }
    }
}
