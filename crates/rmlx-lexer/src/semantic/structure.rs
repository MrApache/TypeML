use crate::{next_or_none, peek_or_none, StructToken};
use crate::semantic::{Attribute, ParserContext};

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
}

impl Struct {
    fn new(name: String, fields: Vec<Field>) -> Self {
        Self {
            name,
            fields,
            attributes: vec![],
        }
    }

    pub(crate) fn set_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes = attributes;
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
    Simple(String),
    Generic(String, String),
    Array(Vec<String>),
    Block(Vec<Field>),
}

impl<'s> ParserContext<'s, StructToken> {
    pub fn parse(&mut self) -> Option<Struct> {
        self.consume_keyword();
        let name = self.consume_type_name()?;
        self.consume_left_curve_brace()?;

        let mut fields = Vec::new();
        loop {
            let next = peek_or_none!(self, "Unexpected end of token stream in struct body")?;
            match next.kind() {
                StructToken::RightCurlyBracket => {
                    let next = next_or_none!(self).unwrap();
                    self.tokens.push(next.to_semantic_token(u32::MAX));
                    break;
                }

                StructToken::Identifier => {
                    fields.push(self.consume_typed_field()?);

                    // после поля может быть , или :
                    let sep = next_or_none!(self, "Unexpected end of token stream after field")?;
                    self.tokens.push(sep.to_semantic_token(u32::MAX));
                    match sep.kind() {
                        StructToken::Comma => continue,
                        StructToken::RightCurlyBracket => break,
                        _ => {
                            self.create_error_message("Expected ',' or '}' after field");
                            return None;
                        }
                    }
                }
                StructToken::NewLine | StructToken::Whitespace => unreachable!(),
                _ => {
                    self.create_error_message("Unexpected token in struct body");
                    return None;
                }
            }
        }

        Some(Struct::new(name, fields))
    }
}
