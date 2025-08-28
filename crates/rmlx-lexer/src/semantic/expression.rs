use crate::{semantic::{Field, ParserContext, Type}, ExpressionToken};

#[derive(Debug)]
pub struct Expression {
    pub name: String,
    pub groups: Vec<String>,
    pub required: Vec<Field>,
    pub additional: Vec<Field>,
    pub available_in: Vec<String>,
}

impl<'s> ParserContext<'s, ExpressionToken> {
    pub fn parse(&mut self) -> Result<Expression, String> {
        self.consume_keyword()?;
        let name = self.consume_type_name()?;
        let mut groups = Vec::new();
        let mut required = Vec::new();
        let mut additional = Vec::new();
        let mut available_in = Vec::new();
        self.consume_left_curve_brace()?;

        loop {
            let next = self.iter.peek().ok_or("Unexpected end of token stream")?;

            match next.kind() {
                ExpressionToken::RightCurlyBracket => {
                    self.tokens.push(next.to_semantic_token(u32::MAX));
                    break;
                }

                ExpressionToken::Identifier => {
                    let field = self.consume_advanced_typed_field()?;
                    match field.name.as_str() {
                        "groups" => {
                            if let Type::Array(arr) = field.ty {
                                groups = arr;
                            } else {
                                return Err("Expected array for `groups`".into());
                            }
                        }
                        "available_in" => {
                            if let Type::Array(arr) = field.ty {
                                available_in = arr;
                            } else {
                                return Err("Expected array for `available_in`".into());
                            }
                        }
                        "required" => {
                            if let Type::Block(block) = field.ty {
                                required = block;
                            } else {
                                return Err("Expected block for `required`".into());
                            }
                        }
                        "additional" => {
                            if let Type::Block(block) = field.ty {
                                additional = block;
                            } else {
                                return Err("Expected block for `additional`".into());
                            }
                        }
                        _ => return Err(format!("Unknown field: {}", field.name)),
                    }
                    // после поля может быть ',' или конец блока '}': съедаем если ','
                    if let Some(tok) = self.iter.peek_mut() {
                        if tok.kind() == &ExpressionToken::Comma {
                            let tok = self.iter.next().unwrap();
                            self.tokens.push(tok.to_semantic_token(u32::MAX));
                        }
                    }
                }
                ExpressionToken::NewLine | ExpressionToken::Whitespace => unreachable!(),
                _ => return Err("Unexpected token in expression body".into()),
            }
        }

        Ok(Expression {
            name,
            groups,
            required,
            additional,
            available_in,
        })
    }
}
