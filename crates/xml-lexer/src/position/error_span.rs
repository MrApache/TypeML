#[derive(PartialEq, Debug)]
pub struct ErrorSpan {
    pub(crate) source: String,
    pub(crate) start: usize,
    pub(crate) length: usize,
}

impl ErrorSpan {
    pub const fn new(source: String, start: usize, length: usize) -> ErrorSpan {
        ErrorSpan {
            source,
            start,
            length,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub const fn start(&self) -> usize {
        self.start
    }

    pub const fn length(&self) -> usize {
        self.length
    }
}
