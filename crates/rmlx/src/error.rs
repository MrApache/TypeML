use logos::{Lexer, Logos};
use lexer_core::Position;

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

impl Error {
    pub(crate) fn from_lexer<'source, T>(lex: &mut Lexer<'source, T>) -> Self
    where
        T: Logos<'source, Extras = Position, Source = str>,
    {
        let ch = lex.slice().chars().next().unwrap();
        Error::UnexpectedChar(ch)
    }
}