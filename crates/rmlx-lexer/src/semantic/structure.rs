use crate::StructToken;
use crate::semantic::ParserContext;

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
    Simple(String),
    Generic(String, String),
    Array(Vec<String>),
    Block(Vec<Field>),
}

impl<'s> ParserContext<'s, StructToken> {
    pub fn parse(&mut self) -> Result<Struct, String> {
        self.consume_keyword()?;
        let name = self.consume_type_name()?;
        self.consume_left_curve_brace()?;

        let mut fields = Vec::new();
        loop {
            let next = self.iter.peek().ok_or("Unexpected EOF in struct body")?;
            match next.kind() {
                StructToken::RightCurlyBracket => {
                    let next = self.iter.next().unwrap();
                    self.tokens.push(next.to_semantic_token(u32::MAX));
                    break;
                }

                StructToken::Identifier => {
                    fields.push(self.consume_typed_field()?);

                    // после поля может быть , или :
                    let sep = self.iter.next().ok_or("Unexpected EOF after field")?;
                    self.tokens.push(sep.to_semantic_token(u32::MAX));
                    match sep.kind() {
                        StructToken::Comma => continue,
                        StructToken::RightCurlyBracket => break,
                        _ => return Err("Expected `,` or `}` after field".into()),
                    }
                }
                StructToken::NewLine | StructToken::Whitespace => unreachable!(),
                _ => return Err("Unexpected token in struct body".into()),
            }
        }

        Ok(Struct { name, fields })
    }
}
