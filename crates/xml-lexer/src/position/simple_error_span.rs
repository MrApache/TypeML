#[derive(PartialEq, Debug)]
pub struct SimpleErrorSpan {
    pub(crate) start: usize,
    pub(crate) length: usize,
}

impl SimpleErrorSpan {
    pub const fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }
}
