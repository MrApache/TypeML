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

    fn identifier() -> Self {unimplemented!()}
    fn colon() -> Self {unimplemented!()}

    fn left_curly_brace() -> Self { unimplemented!() }
    fn right_curly_brace() -> Self { unimplemented!() }
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum SchemaTokens {
    #[token("group", group_callback)]
    Group(Vec<Token<GroupToken>>),

    #[token("element", element_callback)]
    Element(Vec<Token<ElementTokens>>),

    #[token("#", attribute_callback)]
    Attribute(Vec<Token<AttributeToken>>),

    #[token("expression", expression_callback)]
    Expression(Vec<Token<ExpressionTokens>>),

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
    inner: Lexer<'a, SchemaTokens>,
}

impl<'a> RmlxTokenStream<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            inner: SchemaTokens::lexer(content),
        }
    }

    pub fn next_token(&mut self) -> Option<Result<SchemaTokens, Error>> {
        while let Some(token_kind) = self.inner.next() {
            match &token_kind {
                Ok(SchemaTokens::NewLine) => {
                    self.inner.extras.new_line();
                    continue; // пропускаем
                }
                Ok(SchemaTokens::Whitespace) => {
                    self.inner.extras.advance(self.inner.span().len() as u32);
                    continue; // пропускаем
                }
                _ => return Some(token_kind), // значимый токен
            }
        }
        None // конец итератора
    }


    pub fn to_vec(mut self) -> Result<Vec<SchemaTokens>, Error> {
        let mut vec = vec![];

        while let Some(token) = self.next_token() {
            vec.push(token?);
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::RmlxTokenStream;

    #[test]
    fn test() {
        const CONTENT: &str =
            r#"#[Path(std::iter)]"#;

        let _tokens = RmlxTokenStream::new(CONTENT).to_vec();
        println!();
    }
}
