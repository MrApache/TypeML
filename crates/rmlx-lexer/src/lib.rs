mod context;

pub use context::*;

use logos::{Lexer, Logos};
use lexer_utils::*;

use crate::context::*;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum SchemaTokens {
    #[token("group", group_callback)]
    Group(Vec<Token<GroupTokens>>),

    #[token("element", element_callback)]
    Element(Vec<Token<ElementTokens>>),

    #[token("#", attribute_callback)]
    Attribute(Vec<Token<AttributeTokens>>),

    #[token("expression", expression_callback)]
    Expression(Vec<Token<ExpressionTokens>>),

    #[token("enum", enum_callback)]
    Enum(Vec<Token<EnumTokens>>),

    #[token("struct", struct_callback)]
    Struct(Vec<Token<StructTokens>>),

    #[token("use", use_callback)]
    Use(Vec<Token<UseTokens>>),

    #[token("\n")]
    NewLine,

    #[regex(r" \t\r+")]
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
        if let Some(token_kind) = self.inner.next() {
            match &token_kind {
                Ok(SchemaTokens::NewLine) => self.inner.extras.new_line(),
                Ok(SchemaTokens::Whitespace) => self.inner.extras.current_column += 1,
                _ => {}
            }
            Some(token_kind)
        }
        else {
            None
        }
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
