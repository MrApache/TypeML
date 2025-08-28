use lexer_utils::PARAMETER_TOKEN;
use crate::{
    semantic::{parse_attributes, Attribute, ParserContext},
    EnumToken,
};

pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub name: String,
    pub ty:   Option<String>,
    pub pattern: Option<String>,
}

impl<'s> ParserContext<'s, EnumToken> {
    pub fn parse(&mut self) -> Result<Enum, String> {
        self.consume_keyword()?;
        let enum_name = self.consume_type_name()?;
        self.consume_left_curve_brace()?;

        let mut variants = Vec::new();
        let mut attributes = Vec::new();

        // читаем варианты
        loop {
            let token = self
                .iter
                .next()
                .ok_or_else(|| "Unexpected end of tokens while parsing enum".to_string())?;

            match token.kind() {
                // конец enum
                EnumToken::RightCurlyBracket => {
                    self.iter.next(); // съесть '}'
                    self.tokens.push(token.to_semantic_token(u32::MAX));
                    break;
                }

                // атрибуты перед вариантом (0 или более)
                EnumToken::Attribute(tokens) => {
                    attributes = parse_attributes(tokens.iter(), self.src, self.tokens)?;
                }

                // имя варианта
                EnumToken::Identifier => {
                    let name = token.slice(self.src).to_string();
                    self.tokens.push(token.to_semantic_token(PARAMETER_TOKEN));

                    // проверяем, есть ли '(' для аргумента
                    let ty = if let Some(t) = self.iter.clone().next() {
                        if t.kind() == &EnumToken::LeftParenthesis {
                            self.iter.next(); // съесть '('
                            self.tokens.push(t.to_semantic_token(u32::MAX));

                            // читаем тип значения
                            let ty = self.consume_type_name()?;

                            // читаем ')'
                            let t = self.iter.next().ok_or("Expected ')' after type")?;
                            if t.kind() != &EnumToken::RightParenthesis {
                                return Err(format!("Expected ')', got {:?}", t.kind()));
                            }
                            self.tokens.push(t.to_semantic_token(u32::MAX));

                            Some(ty)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let pattern = match attributes.as_slice() {
                        [Attribute::Pattern(value)] => Some(value.value.clone()),
                        [] => None,
                        _ => return Err("Unknown attribute".into()),
                    };


                    variants.push(EnumVariant {
                        name,
                        ty,
                        pattern,
                    });

                    // после варианта может быть ',' или '}'
                    if let Some(t) = self.iter.clone().next() {
                        match t.kind() {
                            EnumToken::Comma => {
                                self.iter.next(); // съесть ','
                                self.tokens.push(t.to_semantic_token(u32::MAX));
                            }
                            EnumToken::RightCurlyBracket => {} // конец enum, обработаем в начале цикла
                            _ => return Err(format!("Expected ',' or '}}', got {:?}", t.kind())),
                        }
                    }
                }
                _ => return Err(format!("Expected variant, got {:?}", token.kind())),
            }
        }

        Ok(Enum {
            name: enum_name,
            variants,
        })
    }
}
