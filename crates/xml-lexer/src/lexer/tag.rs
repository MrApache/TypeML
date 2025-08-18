use crate::lexer::Attribute;
use crate::position::{Location, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub span: Span,
    pub location: Location,
    pub identifier: String,
    pub attributes: Vec<Attribute>,
}

impl Tag {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub const fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    pub const fn location(&self) -> Location {
        self.location
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn into_identifier(self) -> String {
        self.identifier
    }

    pub fn into_attributes(self) -> Vec<Attribute> {
        self.attributes
    }

    pub fn into_inner(self) -> (String, Vec<Attribute>) {
        (self.identifier, self.attributes)
    }
}
