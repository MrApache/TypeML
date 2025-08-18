use std::fmt::Display;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Location {
    pub(crate) line_position: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Display for Location {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}:{}", self.line, self.column)
    }
}

impl Location {
    pub const fn new(position: usize, line: usize, column: usize) -> Self {
        Self {
            line_position: position,
            line,
            column,
        }
    }

    pub const fn position(&self) -> usize {
        self.line_position
    }

    pub const fn line(&self) -> usize {
        self.line
    }

    pub const fn column(&self) -> usize {
        self.column
    }
}
