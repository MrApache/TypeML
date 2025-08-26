mod context;

pub use context::*;

use logos::{Lexer, Logos};
use lexer_utils::*;

use crate::context::*;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum SchemaTokens {
    #[token("group", group_context_callback)]
    Group(Vec<Token<GroupContext>>),

    #[token("element", element_context_callback)]
    Element(Vec<Token<ElementContext>>),

    #[token("#", attribute_context_callback)]
    Attribute(Vec<Token<AttributeContext>>),

    #[token("expression", expression_context_callback)]
    Expression(Vec<Token<ExpressionContext>>),

    #[token("enum", enum_context_callback)]
    Enum(Vec<Token<EnumContext>>),

    #[token("struct", struct_context_callback)]
    Struct(Vec<Token<StructContext>>),

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

    pub fn next_token(&mut self) -> Result<SchemaTokens, ()> {
        if let Some(token_kind) = self.inner.next() {
            match &token_kind {
                Ok(SchemaTokens::NewLine) => self.inner.extras.new_line(),
                Ok(SchemaTokens::Whitespace) => self.inner.extras.current_column += 1,
                _ => {}
            }
            token_kind
        }
        else {
            Err(())
        }
    }

    pub fn to_vec(mut self) -> Vec<SchemaTokens> {
        let mut vec = vec![];

        while let Ok(token) = self.next_token() {
            vec.push(token);
        }

        vec
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
