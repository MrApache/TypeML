use crate::{
    semantic::{Attribute, Field, ParserContext},
    ElementToken,
};

#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub fields: Vec<Field>,
    pub path: Option<String>,
    pub group: Option<String>,
}

impl Element {
    pub fn new(name: String, fields: Vec<Field>) -> Self {
        Self {
            name,
            fields,
            path: None,
            group: None,
        }
    }

    pub fn resolve_attributes(&mut self, attributes: &mut Vec<Attribute>) {
        attributes.retain(|attr| {
            match attr {
                Attribute::Path(path) => self.path = Some(path.value.clone()),
                Attribute::Group(group) => self.group = Some(group.value.clone()),
                _ => return true,
            }

            false
        });
    }
}

impl<'s> ParserContext<'s, ElementToken> {
    pub fn parse(&mut self) -> Result<Element, String> {
        self.consume_keyword()?;
        let name = self.consume_type_name()?;
        let mut fields = Vec::new();

        let next = self.iter.peek().ok_or("Unexpected EOF in element body")?;
        match next.kind() {
            ElementToken::Semicolon => {
                let next = self.iter.next().unwrap();
                self.tokens.push(next.to_semantic_token(u32::MAX));
            }
            ElementToken::LeftCurlyBracket => {
                self.consume_left_curve_brace()?;

                loop {
                    let next = self.iter.peek().ok_or("Unexpected EOF in element body")?;
                    match next.kind() {
                        ElementToken::RightCurlyBracket | ElementToken::Semicolon => {
                            let next = self.iter.next().unwrap();
                            self.tokens.push(next.to_semantic_token(u32::MAX));
                            break;
                        }

                        ElementToken::Identifier => {
                            fields.push(self.consume_typed_field()?);

                            // после поля может быть , или :
                            let sep = self.iter.next().ok_or("Unexpected EOF after field")?;
                            self.tokens.push(sep.to_semantic_token(u32::MAX));
                            match sep.kind() {
                                ElementToken::Comma => continue,
                                ElementToken::RightCurlyBracket => break,
                                _ => return Err("Expected `,` or `}` after field".into()),
                            }
                        }
                        ElementToken::NewLine | ElementToken::Whitespace => unreachable!(),
                        _ => return Err("Unexpected token in element body".into()),
                    }
                }
            }
            _ => return Err("Unexpected token in element body".into()),
        }

        Ok(Element::new(name, fields))
    }
}
