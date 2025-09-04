use crate::{
    ast::{Field, ParserContext},
    next_or_none, peek_or_none, Attribute, ExpressionToken,
};

#[derive(Debug)]
pub struct Expression {
    name: String,
    fields: Vec<Field>,
    attributes: Vec<Attribute>,
}

impl Expression {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

impl ParserContext<'_, ExpressionToken> {
    pub fn parse(&mut self) -> Option<Expression> {
        self.consume_keyword();
        let name = self.consume_type_name()?;
        let mut fields = vec![];
        self.consume_left_curve_brace()?;

        loop {
            let next = peek_or_none!(self)?;
            match next.kind() {
                ExpressionToken::Identifier => {
                    let field = self.consume_advanced_typed_field()?;
                    fields.push(field);

                    if let Some(token) = self.iter.peek() {
                        match token.kind() {
                            ExpressionToken::Comma => { next_or_none!(self).unwrap(); }
                            ExpressionToken::SyntaxError => self.consume_error("Syntax error"),
                            _ => {}
                        }
                    }
                }
                ExpressionToken::RightCurlyBracket => break,
                ExpressionToken::NewLine | ExpressionToken::Whitespace => unreachable!(),
                ExpressionToken::SyntaxError => self.consume_error("Syntax error"),
                _ => self.consume_error("Unexpected token in expression body"),
            }
        }

        Some(Expression {
            name,
            fields,
            attributes: vec![],
        })
    }
}
