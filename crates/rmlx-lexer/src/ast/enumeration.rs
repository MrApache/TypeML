use crate::{
    ast::{Attribute, ParserContext},
    next_or_none, peek_or_none, EnumToken,
};
use lexer_utils::PARAMETER_TOKEN;

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub ty: Option<String>,
    pub attributes: Vec<Attribute>,
}

impl Enum {
    pub fn new(name: String, variants: Vec<EnumVariant>) -> Self {
        Self {
            name,
            variants,
            attributes: vec![],
        }
    }

    pub(crate) fn set_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes = attributes;
    }
}

impl<'s> ParserContext<'s, EnumToken> {
    pub fn parse(&mut self) -> Option<Enum> {
        self.consume_keyword();
        let enum_name = self.consume_type_name()?;
        self.consume_left_curve_brace()?;

        let mut variants = Vec::new();
        let mut attributes = Vec::new();

        loop {
            let token = next_or_none!(self, "Unexpected end of tokens while parsing enum")?;
            match token.kind() {
                EnumToken::RightCurlyBracket => {
                    self.tokens.push(token.to_semantic_token(u32::MAX));
                    break;
                }

                EnumToken::Attribute(tokens) => {
                    attributes = ParserContext::new(
                        tokens.iter().peekable(),
                        self.diagnostics,
                        self.tokens,
                        self.src,
                    )
                    .parse()
                    .unwrap_or_default();
                }

                EnumToken::Identifier => {
                    let name = token.slice(self.src).to_string();
                    self.tokens.push(token.to_semantic_token(PARAMETER_TOKEN));

                    let ty = if let Some(t) = self.iter.peek() {
                        if t.kind() == &EnumToken::LeftParenthesis {
                            let t = next_or_none!(self).unwrap(); // съесть '('
                            self.tokens.push(t.to_semantic_token(u32::MAX));

                            let ty = self.consume_type_name()?;

                            let t = next_or_none!(self, "Expected ')' after type")?;
                            match t.kind() {
                                EnumToken::LeftParenthesis => {
                                    self.tokens.push(t.to_semantic_token(u32::MAX))
                                }
                                EnumToken::SyntaxError => self.report_error(t, "Syntax error"),
                                kind => {
                                    self.report_error(t, &format!("Expected ')', got {kind}",));
                                }
                            }
                            Some(ty)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    variants.push(EnumVariant {
                        name,
                        ty,
                        attributes: std::mem::take(&mut attributes),
                    });

                    if let Some(t) = peek_or_none!(self) {
                        match t.kind() {
                            EnumToken::Comma => {
                                let t = next_or_none!(self).unwrap();
                                self.tokens.push(t.to_semantic_token(u32::MAX));
                            }
                            EnumToken::RightCurlyBracket => continue, // конец enum, обработаем в начале цикла
                            EnumToken::SyntaxError => self.consume_error("Syntax error"),
                            kind => {
                                self.consume_error(&format!("Expected ',' or '}}', got {kind}"))
                            }
                        }
                    }
                }
                EnumToken::SyntaxError => self.report_error(token, "Syntax error"),
                kind => self.report_error(token, &format!("Expected variant, got {kind}")),
            }
        }

        Some(Enum::new(enum_name, variants))
    }
}
