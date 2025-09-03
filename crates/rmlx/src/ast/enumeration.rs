use crate::{
    ast::{Attribute, ParserContext},
    next_or_none, EnumDefinitionToken, RuleToken,
};
use lexer_core::{Token, PARAMETER_TOKEN, STRING_TOKEN};

#[derive(Debug)]
pub struct Enum {
    name: String,
    variants: Vec<EnumVariant>,
    attributes: Vec<Attribute>,
}

impl Enum {
    #[must_use]
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

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn variants(&self) -> &[EnumVariant] {
        &self.variants
    }

    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

#[derive(Debug)]
pub struct EnumVariant {
    name: String,
    ty: Option<String>,
    pattern: Option<String>,
    attributes: Vec<Attribute>,
}

impl EnumVariant {
    #[must_use]
    pub fn new(
        name: String,
        ty: Option<String>,
        pattern: Option<String>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            name,
            ty,
            pattern,
            attributes,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn ty(&self) -> Option<&str> {
        self.ty.as_deref()
    }

    #[must_use]
    pub fn pattern(&self) -> Option<&str> {
        self.pattern.as_deref()
    }

    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

impl ParserContext<'_, EnumDefinitionToken> {
    pub fn parse(&mut self) -> Option<Enum> {
        self.consume_keyword();
        let enum_name = self.consume_type_name()?;
        self.consume_left_curve_brace()?;

        let mut variants = Vec::new();
        let mut attributes = Vec::new();

        loop {
            let token = next_or_none!(self, "Unexpected end of tokens while parsing enum")?;
            match token.kind() {
                EnumDefinitionToken::Comma => {
                    self.tokens.push(token.to_semantic_token(u32::MAX));
                }

                EnumDefinitionToken::RightCurlyBracket => {
                    self.tokens.push(token.to_semantic_token(u32::MAX));
                    break;
                }

                EnumDefinitionToken::Attribute(tokens) => {
                    attributes = ParserContext::new(
                        tokens.iter().peekable(),
                        self.diagnostics,
                        self.tokens,
                        self.src,
                    )
                    .parse()
                    .unwrap_or_default();
                }

                EnumDefinitionToken::Identifier => {
                    let mut inner_type = None;

                    let name = token.slice(self.src).to_string();
                    self.tokens.push(token.to_semantic_token(PARAMETER_TOKEN));

                    if let Some(peek) = self.iter.peek() {
                        match peek.kind() {
                            EnumDefinitionToken::LeftParenthesis => {
                                self.consume_left_par();
                                let type_name = self.consume_type_name()?;
                                self.consume_right_par()?;
                                inner_type = Some(type_name);
                            }
                            EnumDefinitionToken::Rule(tokens) => {
                                variants.push(EnumVariant::new(name.clone(), None, self.consume_rule(tokens), std::mem::take(&mut attributes)));
                            }
                            EnumDefinitionToken::RightCurlyBracket => {} // continue
                            EnumDefinitionToken::Comma => self.consume_comma(),
                            EnumDefinitionToken::SyntaxError => self.consume_error("Syntax error"),
                            kind => self.consume_error(&format!("Expected ',' or '}}', got {kind}")),
                        }
                    } else {
                        variants.push(EnumVariant::new(name.clone(), None, None, std::mem::take(&mut attributes)));
                    }

                    if let Some(peek) = self.iter.peek() {
                        if inner_type.is_some() && let EnumDefinitionToken::Rule(tokens) = peek.kind() {
                            variants.push(EnumVariant::new(name.clone(), inner_type, self.consume_rule(tokens), std::mem::take(&mut attributes)));
                        }
                    }
                    else {
                        variants.push(EnumVariant::new(name, inner_type, None, std::mem::take(&mut attributes)));
                    }
                }
                EnumDefinitionToken::SyntaxError => self.report_error(token, "Syntax error"),
                kind => self.report_error(token, &format!("Expected variant, got {kind}")),
            }
        }

        Some(Enum::new(enum_name, variants))
    }

    fn consume_left_par(&mut self) {
        let left_par = next_or_none!(self).unwrap();
        self.tokens.push(left_par.to_semantic_token(u32::MAX));
    }

    fn consume_right_par(&mut self) -> Option<()> {
        let right_par = next_or_none!(self, "Expected ')' after type")?;
        self.tokens.push(right_par.to_semantic_token(u32::MAX));
        match right_par.kind() {
            EnumDefinitionToken::RightParenthesis => return Some(()),
            EnumDefinitionToken::SyntaxError => self.report_error_message("Syntax error"),
            kind => self.report_error_message(&format!("Expected ')', got {kind}",)),
        }

        None
    }

    fn consume_rule(&mut self, tokens: &[Token<RuleToken>]) -> Option<String> {
        next_or_none!(self).unwrap();
        ParserContext::new(tokens.iter().peekable(), self.diagnostics, self.tokens, self.src).parse()
    }

    fn consume_comma(&mut self) {
        let comma = next_or_none!(self).unwrap();
        self.tokens.push(comma.to_semantic_token(u32::MAX));
    }
}

impl ParserContext<'_, RuleToken> {
    pub fn parse(&mut self) -> Option<String> {
        self.consume_pipe();
        self.consume_string()
    }

    fn consume_pipe(&mut self) {
        let pipe = next_or_none!(self).unwrap();
        self.tokens.push(pipe.to_semantic_token(u32::MAX));
    }

    fn consume_string(&mut self) -> Option<String> {
        let string = next_or_none!(self)?;
        if string.kind() != &RuleToken::String {
            self.tokens.push(string.to_semantic_token(u32::MAX));
            return None;
        }
        self.tokens.push(string.to_semantic_token(STRING_TOKEN));
        Some(string.slice(self.src).to_string())
    }
}
