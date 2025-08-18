use crate::position::{Location, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value {
    pub(crate) span: Span,
    pub(crate) location: Location,
    pub(crate) inner: String,
}

impl Value {
    pub fn new(span: Span, location: Location, inner: impl Into<String>) -> Self {
        Self {
            span,
            location,
            inner: inner.into(),
        }
    }
    pub fn value(&self) -> &str {
        &self.inner
    }

    pub const fn location(&self) -> Location {
        self.location
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn into_inner(self) -> String {
        self.inner
    }
}
