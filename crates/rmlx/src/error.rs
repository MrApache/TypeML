#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[default]
    UnknownChar,
    UnexpectedChar(char),

    MissingOpeningBrace,

    UnexpectedToken {
        expected: Vec<&'static str>,
        found: String,
    },
    MissingToken {
        expected: Vec<&'static str>,
    },
}
