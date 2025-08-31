use crate::{peek_or_none, ast::{Field, ParserContext, Type}, ExpressionToken};

#[derive(Debug)]
pub struct Expression {
    pub name: String,
    pub groups: Vec<String>,
    pub required: Vec<Field>,
    pub additional: Vec<Field>,
    pub available_in: Vec<String>,
}

impl<'s> ParserContext<'s, ExpressionToken> {
    pub fn parse(&mut self) -> Option<Expression> {
        self.consume_keyword();
        let name = self.consume_type_name()?;
        let mut groups = Vec::new();
        let mut required = Vec::new();
        let mut additional = Vec::new();
        let mut available_in = Vec::new();
        self.consume_left_curve_brace()?;

        loop {
            let next = peek_or_none!(self)?;
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
                                self.create_error_message("Expected array for 'groups'");
                                return None;
                            }
                        }
                        "available_in" => {
                            if let Type::Array(arr) = field.ty {
                                available_in = arr;
                            } else {
                                self.create_error_message("Expected array for `available_in`");
                                return None;
                            }
                        }
                        "required" => {
                            if let Type::Block(block) = field.ty {
                                required = block;
                            } else {
                                self.create_error_message("Expected block for `required`");
                                return None;
                            }
                        }
                        "additional" => {
                            if let Type::Block(block) = field.ty {
                                additional = block;
                            } else {
                                self.create_error_message("Expected block for `additional`");
                                return None;
                            }
                        }
                        _ => {
                            self.create_error_message(format!("Unknown field: {}", field.name));
                            return None;
                        }
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
                _ => {
                    self.create_error_message("Unexpected token in expression body");
                    return None;
                }
            }
        }

        Some(Expression {
            name,
            groups,
            required,
            additional,
            available_in,
        })
    }
}
