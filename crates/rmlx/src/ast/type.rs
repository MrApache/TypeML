use crate::{next_or_none, peek_or_none, Attribute, Field, ParserContext, TypeDefinitionToken};

#[derive(Debug)]
pub struct RefType {
    namespace: Option<String>,
    name: String,
}

impl RefType {
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct TypeDefinition {
    keyword: String,
    name: String,
    fields: Vec<Field>,
    bind: Option<RefType>,
    pub(crate) attributes: Vec<Attribute>,
}

impl TypeDefinition {
    #[must_use]
    pub fn keyword(&self) -> &str {
        &self.keyword
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    #[must_use]
    pub fn bind(&self) -> Option<&RefType> {
        self.bind.as_ref()
    }

    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

impl ParserContext<'_, TypeDefinitionToken> {
    pub fn parse(&mut self) -> Option<TypeDefinition> {
        let keyword = self.consume_keyword().to_string();
        let name = self.consume_type_name()?;
        let bind = self.try_parse_binding();
        let mut fields = Vec::new();

        let next = peek_or_none!(self)?;
        match next.kind() {
            TypeDefinitionToken::Semicolon => { next_or_none!(self).unwrap(); }
            TypeDefinitionToken::LeftCurlyBracket => {
                self.consume_left_curve_brace()?;

                loop {
                    let next = peek_or_none!(
                        self,
                        "Unexpected end of token stream in type definition body"
                    )?;
                    match next.kind() {
                        TypeDefinitionToken::RightCurlyBracket | TypeDefinitionToken::Semicolon => {
                            next_or_none!(self).unwrap();
                            break;
                        }

                        TypeDefinitionToken::Identifier => {
                            fields.push(self.consume_typed_field()?);

                            let sep = next_or_none!(self, "Unexpected end of token stream after field")?;
                            match sep.kind() {
                                TypeDefinitionToken::Comma => {}, // continue
                                TypeDefinitionToken::RightCurlyBracket => break,
                                TypeDefinitionToken::SyntaxError => self.report_error("Syntax error"),
                                _ => self.report_error("Expected ',' or '}' after field"),
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

        Some(TypeDefinition {
            keyword,
            name,
            fields,
            bind,
            attributes: vec![],
        })
    }

    fn try_parse_binding(&mut self) -> Option<RefType> {
        let dash = peek_or_none!(self)?;
        match dash.kind() {
            TypeDefinitionToken::Dash => { next_or_none!(self).unwrap(); }
            TypeDefinitionToken::SyntaxError => {
                self.consume_error("Syntax error");
                return None;
            }
            _ => return None,
        }

        let right_angle_bracket = next_or_none!(self)?;
        match right_angle_bracket.kind() {
            TypeDefinitionToken::RightAngleBracket => {}
            TypeDefinitionToken::SyntaxError => {
                self.report_error("Syntax error");
                return None;
            }
            kind => {
                self.report_error(&format!("Expected '>', got {kind}"));
                return None;
            }
        }

        let identifier = next_or_none!(self)?;
        match identifier.kind() {
            TypeDefinitionToken::Identifier => {
                Some(RefType {
                    namespace: None, //TODO namespace
                    name: identifier.slice(self.src).to_string(),
                })
            }
            TypeDefinitionToken::SyntaxError => {
                self.report_error("Syntax error");
                None
            }
            kind => {
                self.report_error(&format!("Expected identifier, got {kind}"));
                None
            }
        }
    }
}
