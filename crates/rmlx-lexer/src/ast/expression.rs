use crate::{
    ast::{Field, ParserContext},
    next_or_none, peek_or_none, ExpressionToken,
};

#[derive(Debug)]
pub struct Expression {
    pub name: String,
    pub fields: Vec<Field>,
}

impl<'s> ParserContext<'s, ExpressionToken> {
    pub fn parse(&mut self) -> Option<Expression> {
        self.consume_keyword();
        let name = self.consume_type_name()?;
        let mut fields = vec![];
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
                    fields.push(field);

                    if let Some(token) = self.iter.peek() {
                        match token.kind() {
                            ExpressionToken::Comma => {
                                let token = next_or_none!(self).unwrap();
                                self.tokens.push(token.to_semantic_token(u32::MAX));
                            }
                            ExpressionToken::SyntaxError => self.consume_error("Syntax error"),
                            _ => {}
                        }
                    }
                }
                ExpressionToken::NewLine | ExpressionToken::Whitespace => unreachable!(),
                ExpressionToken::SyntaxError => self.consume_error("Syntax error"),
                _ => self.consume_error("Unexpected token in expression body"),
            }
        }

        Some(Expression { name, fields })
    }
}
