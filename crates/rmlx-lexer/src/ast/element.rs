use crate::{
    ast::{Attribute, Field, ParserContext},
    next_or_none, peek_or_none, ElementToken,
};

#[derive(Debug)]
pub struct Element {
    pub name: String,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
}

impl Element {
    pub fn new(name: String, fields: Vec<Field>) -> Self {
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

impl<'s> ParserContext<'s, ElementToken> {
    pub fn parse(&mut self) -> Option<Element> {
        self.consume_keyword();
        let name = self.consume_type_name()?;
        let mut fields = Vec::new();

        let next = peek_or_none!(self)?;
        match next.kind() {
            ElementToken::Semicolon => {
                let next = next_or_none!(self).unwrap();
                self.tokens.push(next.to_semantic_token(u32::MAX));
            }
            ElementToken::LeftCurlyBracket => {
                self.consume_left_curve_brace()?;

                loop {
                    let next = peek_or_none!(self, "Unexpected end of token stream in element body")?;
                    match next.kind() {
                        ElementToken::RightCurlyBracket | ElementToken::Semicolon => {
                            let next = next_or_none!(self).unwrap();
                            self.tokens.push(next.to_semantic_token(u32::MAX));
                            break;
                        }

                        ElementToken::Identifier => {
                            fields.push(self.consume_typed_field()?);

                            let sep = next_or_none!(self, "Unexpected end of token stream after field")?;
                            self.tokens.push(sep.to_semantic_token(u32::MAX));
                            match sep.kind() {
                                ElementToken::Comma => continue,
                                ElementToken::RightCurlyBracket => break,
                                ElementToken::SyntaxError => self.report_error(sep, "Syntax error"),
                                _ => self.report_error(sep, "Expected ',' or '}' after field"),
                            }
                        }
                        ElementToken::NewLine | ElementToken::Whitespace => unreachable!(),
                        ElementToken::SyntaxError => self.consume_error("Syntax error"),
                        _ => self.consume_error("Unexpected token in element body"),
                    }
                }
            }
            ElementToken::SyntaxError => self.consume_error("Syntax error"),
            _ => self.consume_error("Unexpected token in element body"),
        }

        Some(Element::new(name, fields))
    }
}
