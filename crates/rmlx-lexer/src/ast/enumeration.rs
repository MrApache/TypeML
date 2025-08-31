use crate::{
    next_or_none, peek_or_none, ast::{Attribute, ParserContext}, EnumToken
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

        // читаем варианты
        loop {
            let token = next_or_none!(self, "Unexpected end of tokens while parsing enum")?;
            match token.kind() {
                // конец enum
                EnumToken::RightCurlyBracket => {
                    self.tokens.push(token.to_semantic_token(u32::MAX));
                    break;
                }

                // атрибуты перед вариантом (0 или более)
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

                // имя варианта
                EnumToken::Identifier => {
                    let name = token.slice(self.src).to_string();
                    self.tokens.push(token.to_semantic_token(PARAMETER_TOKEN));

                    // проверяем, есть ли '(' для аргумента
                    let ty = if let Some(t) = self.iter.peek() {
                        if t.kind() == &EnumToken::LeftParenthesis {
                            let t = next_or_none!(self).unwrap(); // съесть '('
                            self.tokens.push(t.to_semantic_token(u32::MAX));

                            // читаем тип значения
                            let ty = self.consume_type_name()?;

                            // читаем ')'
                            let t = next_or_none!(self, "Expected ')' after type")?;
                            if t.kind() != &EnumToken::RightParenthesis {
                                self.create_error_message(format!(
                                    "Expected ')', got {:?}",
                                    t.kind()
                                ));
                                return None;
                            }
                            self.tokens.push(t.to_semantic_token(u32::MAX));

                            Some(ty)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    variants.push(EnumVariant { name, ty, attributes: std::mem::take(&mut attributes) });

                    // после варианта может быть ',' или '}'
                    if let Some(t) = peek_or_none!(self) {
                        match t.kind() {
                            EnumToken::Comma => {
                                let t = next_or_none!(self).unwrap();
                                self.tokens.push(t.to_semantic_token(u32::MAX));
                            }
                            EnumToken::RightCurlyBracket => continue, // конец enum, обработаем в начале цикла
                            kind => {
                                self.create_error_message(format!("Expected ',' or '}}', got {kind}"));
                                return None;
                            }
                        }
                    }
                }
                kind => {
                    self.create_error_message(format!("Expected variant, got {kind}"));
                    return None;
                }
            }
        }

        Some(Enum::new(enum_name, variants))
    }
}
