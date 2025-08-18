use crate::errors::{ErrorContext, XmlLayoutError};
use crate::layout_reader::XmlLexer;
use crate::position::Location;

impl<'a> XmlLexer<'a> {
    pub(super) fn err_unexpected_eof(&self) -> XmlLayoutError {
        XmlLayoutError::EndOfFile {
            file: self.file.clone(),
            location: self.location(),
        }
    }

    pub(super) fn err_unexpected_char(&self, expected: char, found: char) -> XmlLayoutError {
        XmlLayoutError::UnexpectedChar {
            context: ErrorContext::new(self.file.clone(), self.location(), self.error_span(1)),
            expected,
            found,
        }
    }

    pub(super) fn err_unexpected_char_with_loc(
        &self,
        location: Location,
        expected: char,
        found: char,
    ) -> XmlLayoutError {
        XmlLayoutError::UnexpectedChar {
            context: ErrorContext::new(self.file.clone(), location, self.error_span(1)),
            expected,
            found,
        }
    }

    pub(super) fn err_expected_identifier(&self, found: char) -> XmlLayoutError {
        XmlLayoutError::ExpectedIdentifier {
            context: ErrorContext::new(self.file.clone(), self.location(), self.error_span(1)),
            found,
        }
    }

    pub(super) fn err_invalid_char(&self, found: char) -> XmlLayoutError {
        XmlLayoutError::InvalidChar {
            context: ErrorContext::new(self.file.clone(), self.location(), self.error_span(1)),
            char: found,
        }
    }
}
