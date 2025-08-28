mod context;
mod semantic_model;
mod semantic;

use std::fmt::Display;

pub use context::*;

use logos::{Lexer, Logos};
use lexer_utils::*;
use crate::context::*;

pub trait TokenDefinition: PartialEq + Eq + Sized + Display {
    fn keyword() -> &'static str;
    fn keyword_token() -> Self;
}

pub trait TokenBodyStatement: PartialEq + Eq + Sized + Display {
    fn left_curly_bracket() -> Self;
    fn right_curly_bracket() -> Self;
}

pub trait TokenSimpleTypeProvider: NamedStatement {
    fn colon() -> Self;
    fn left_angle_bracket() -> Self;
    fn right_angle_bracket() -> Self;
}

pub trait TokenArrayProvider: NamedStatement {
    fn comma() -> Self;
    fn left_square_bracket() -> Self;
    fn right_square_bracket() -> Self;
}

pub trait NamedStatement: PartialEq + Eq + Sized + Display {
    fn identifier() -> Self;
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum SchemaStatement {
    #[token("group", group_callback)]
    Group(Vec<Token<GroupToken>>),

    #[token("element", element_callback)]
    Element(Vec<Token<ElementToken>>),

    #[token("#", attribute_callback)]
    Attribute(Vec<Token<AttributeToken>>),

    #[token("expression", expression_callback)]
    Expression(Vec<Token<ExpressionToken>>),

    #[token("enum", enum_callback)]
    Enum(Vec<Token<EnumToken>>),

    #[token("struct", struct_callback)]
    Struct(Vec<Token<StructToken>>),

    #[token("use", use_callback)]
    Use(Vec<Token<UseToken>>),

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace
}

pub struct RmlxTokenStream<'a> {
    inner: Lexer<'a, SchemaStatement>,
}

impl<'a> RmlxTokenStream<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            inner: SchemaStatement::lexer(content),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<SchemaStatement, Error>> {
        while let Some(token_kind) = self.inner.next() {
            match &token_kind {
                Ok(SchemaStatement::NewLine) => self.inner.extras.new_line(),
                Ok(SchemaStatement::Whitespace) => self.inner.extras.advance(self.inner.span().len() as u32),
                _ => return Some(token_kind), // значимый токен
            }
        }
        None // конец итератора
    }


    pub fn to_vec(mut self) -> Result<Vec<SchemaStatement>, Error> {
        let mut vec = vec![];

        while let Some(token) = self.next_token() {
            vec.push(token?);
        }

        Ok(vec)
    }
}
