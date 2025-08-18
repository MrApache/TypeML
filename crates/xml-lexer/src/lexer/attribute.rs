use crate::lexer::value::Value;
use crate::position::{Location, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub(crate) span: Span,
    pub(crate) location: Location,
    pub(crate) name: Value,
    pub(crate) value: Value,
}

impl Attribute {
    pub fn name(&self) -> &str {
        self.name.value()
    }

    pub fn value(&self) -> &str {
        self.value.value()
    }

    pub const fn location(&self) -> Location {
        self.location
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn into_name(self) -> String {
        self.name.into_inner()
    }

    pub fn into_value(self) -> String {
        self.value.into_inner()
    }
}
