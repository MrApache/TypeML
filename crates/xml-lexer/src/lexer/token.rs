use crate::lexer::Tag;
use crate::lexer::tag_end::TagEnd;
use std::fmt;

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Token {
    TagStart(Tag),
    TagEmpty(Tag),
    TagEnd(TagEnd),
    Text(String),
    Comment,
    #[default]
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::TagStart(_) => write!(f, "TagStart"),
            Token::TagEnd(_) => write!(f, "TagEnd"),
            Token::TagEmpty(_) => write!(f, "TagEmpty"),
            Token::Text(_) => write!(f, "Text"),
            Token::Comment => write!(f, "Comment"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}
