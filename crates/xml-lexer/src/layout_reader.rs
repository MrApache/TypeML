use crate::position::{Location, Span};
use std::io::Cursor;

pub struct XmlLexer<'a> {
    pub(crate) file: String,
    pub(crate) inner: Cursor<&'a str>,
    pub(crate) location: Location,
    pub(crate) start_of_line: usize,
    pub(crate) current_span: Span,
}

impl<'a> XmlLexer<'a> {
    pub fn new(content: &'a str, file: &'a str) -> Self {
        Self {
            file: String::from(file),
            inner: Cursor::new(content),
            location: Location::new(1, 1, 0),
            current_span: Span::new(0, 0),
            start_of_line: 1,
        }
    }
}
