use crate::{next_or_none, peek_or_none, Attribute, Field, ParserContext, TypeDefinitionToken};

#[derive(Debug)]
pub struct Type {
    pub keyword: String,
    pub name: String,
    pub fields: Vec<Field>,
    pub attributes: Vec<Attribute>,
}

impl Type {
    pub fn new(keyword: String, name: String, fields: Vec<Field>) -> Self {
        Self {
            keyword,
            name,
            fields,
            attributes: vec![],
        }
    }

    pub(crate) fn set_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes = attributes;
    }
}

impl<'s> ParserContext<'s, TypeDefinitionToken> {
    pub fn parse(&mut self) -> Option<Type> {
        let keyword = self.consume_keyword().to_string();
        let name = self.consume_type_name()?;
        let mut fields = Vec::new();

        let next = peek_or_none!(self)?;
        match next.kind() {
            TypeDefinitionToken::Semicolon => {
                let next = next_or_none!(self).unwrap();
                self.tokens.push(next.to_semantic_token(u32::MAX));
            }
            TypeDefinitionToken::LeftCurlyBracket => {
                self.consume_left_curve_brace()?;

                loop {
                    let next = peek_or_none!(
                        self,
                        "Unexpected end of token stream in type definition body"
                    )?;
                    match next.kind() {
                        TypeDefinitionToken::RightCurlyBracket | TypeDefinitionToken::Semicolon => {
                            let next = next_or_none!(self).unwrap();
                            self.tokens.push(next.to_semantic_token(u32::MAX));
                            break;
                        }

                        TypeDefinitionToken::Identifier => {
                            fields.push(self.consume_typed_field()?);

                            let sep =
                                next_or_none!(self, "Unexpected end of token stream after field")?;
                            self.tokens.push(sep.to_semantic_token(u32::MAX));
                            match sep.kind() {
                                TypeDefinitionToken::Comma => continue,
                                TypeDefinitionToken::RightCurlyBracket => break,
                                TypeDefinitionToken::SyntaxError => {
                                    self.report_error(sep, "Syntax error")
                                }
                                _ => self.report_error(sep, "Expected ',' or '}' after field"),
                            }
                        }
                        TypeDefinitionToken::NewLine | TypeDefinitionToken::Whitespace => {
                            unreachable!()
                        }
                        TypeDefinitionToken::SyntaxError => self.consume_error("Syntax error"),
                        _ => self.consume_error("Unexpected token in type definition body"),
                    }
                }
            }
            TypeDefinitionToken::SyntaxError => self.consume_error("Syntax error"),
            _ => self.consume_error("Unexpected token in type definition body"),
        }

        Some(Type::new(keyword, name, fields))
    }
}
