use crate::{
    next_or_none, peek_or_none,
    ast::{Attribute, ParserContext},
    GroupToken,
};

#[derive(Debug)]
pub struct Group {
    name: String,
    groups: Vec<String>,
    attributes: Vec<Attribute>,
}

impl Group {
    fn new(name: String, groups: Vec<String>) -> Self {
        Self {
            name,
            groups,
            attributes: vec![],
        }
    }

    pub fn set_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes = attributes
    }
}

impl<'s> ParserContext<'s, GroupToken> {
    pub fn parse(&mut self) -> Option<Group> {
        self.consume_keyword();
        let name = self.consume_type_name()?;

        let t = peek_or_none!(self, "Expected ';' or '[' after identifier")?;

        match t.kind() {
            GroupToken::Semicolon => {
                let t = next_or_none!(self).unwrap();
                self.tokens.push(t.to_semantic_token(u32::MAX));
                Some(Group::new(name, vec![]))
            }
            GroupToken::LeftSquareBracket => {
                let groups = self.consume_array().unwrap_or_default();

                let t = next_or_none!(self, "Expected ';' after group declaration")?;
                match t.kind() {
                    GroupToken::Semicolon => self.tokens.push(t.to_semantic_token(u32::MAX)),
                    GroupToken::SyntaxError => self.report_error(t, "Syntax error"),
                    kind =>  {
                        self.report_error(t, &format!("Expected ';' after group declaration, got {kind}"));
                    }
                }
                Some(Group::new(name, groups))
            }
            GroupToken::SyntaxError => {
                self.consume_error("Syntax error");
                None
            }
            kind => {
                self.consume_error(&format!("Expected ';' or '[', got {kind}"));
                None
            }
        }
    }
}
