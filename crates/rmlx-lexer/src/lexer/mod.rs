mod group;
mod attribute;
mod expression;
mod enumeration;
mod r#use;
mod r#type;

pub use group::*;
pub use attribute::*;
pub use expression::*;
pub use enumeration::*;
pub use r#use::*;
pub use r#type::*;

use lexer_utils::Position;
use logos::{Lexer, Logos};

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
