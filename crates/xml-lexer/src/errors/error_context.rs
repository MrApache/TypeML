use crate::position::{ErrorSpan, Location};
use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
pub struct ErrorContext {
    pub file: String,
    pub location: Location,
    pub error: ErrorSpan,
}

impl ErrorContext {
    pub const fn new(file: String, location: Location, error: ErrorSpan) -> Self {
        Self {
            file,
            location,
            error,
        }
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.location)
    }
}
