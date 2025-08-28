use crate::{
    semantic::{Attribute, ParserContext},
    GroupToken,
};
use lexer_utils::TYPE_TOKEN;

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
                _ => return true,
            }

            false
        });
    }
}

impl<'s> ParserContext<'s, GroupToken> {
    pub fn parse(&mut self) -> Result<Group, String> {
        self.consume_keyword()?;
        let name = self.consume_type_name()?;

        let t = self
            .iter
            .peek()
            .ok_or("Expected `;` or `[` after identifier")?;

        match t.kind() {
            GroupToken::Semicolon => {
                let t = self.iter.next().unwrap();
                self.tokens.push(t.to_semantic_token(u32::MAX));
                Ok(Group::new(name, vec![]))
            }
            GroupToken::LeftSquareBracket => {
                let groups = self.consume_array()?;

                // после `]` обязательно `;`
                let t = self
                    .iter
                    .next()
                    .ok_or("Expected `;` after group declaration")?;

                if t.kind() != &GroupToken::Semicolon {
                    return Err(format!(
                        "Expected `;` after group declaration, got {:?}",
                        t.kind()
                    ));
                }
                self.tokens.push(t.to_semantic_token(u32::MAX));

                Ok(Group::new(name, groups))
            }
            _ => Err(format!("Expected `;` or `[`, got {:?}", t.kind())),
        }
    }
}
