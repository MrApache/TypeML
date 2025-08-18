use crate::position::Span;

#[derive(Debug, PartialEq, Eq)]
pub struct TagEnd {
    pub span: Span,
    pub identifier: String,
}

impl TagEnd {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}
