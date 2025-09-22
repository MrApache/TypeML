use crate::{
    ast::{Attribute, ParserContext},
    next_or_none, peek_or_none, GroupDefinitionToken, QuantifierToken,
};
use lexer_core::{KEYWORD_TOKEN, MACRO_TOKEN, TYPE_TOKEN};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Count {
    Range(Range<u32>),
    Single(u32),
}

#[derive(Debug)]
pub struct RefGroup {
    namespace: Option<String>,
    name: String,
    count: Option<Count>,
    unique: bool,
}

impl RefGroup {
    #[must_use]
    pub fn namespace(&self) -> &Option<String> {
        &self.namespace
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    
    #[must_use]
    pub fn count(&self) -> &Option<Count> {
        &self.count
    }

    #[must_use]
    pub fn unique(&self) -> bool {
        self.unique
    }
}

#[derive(Debug)]
pub struct Group {
    name: String,
    groups: Vec<RefGroup>,
    count: Option<Count>,
    attributes: Vec<Attribute>,
}

impl Group {
    fn new(name: String, groups: Vec<RefGroup>, count: Option<Count>) -> Self {
        Self {
            name,
            groups,
            attributes: vec![],
            count,
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
    pub fn groups(&self) -> &[RefGroup] {
        &self.groups
    }

    #[must_use]
    pub fn count(&self) -> &Option<Count> {
        &self.count
    }

    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

enum PipeIdentifier {
    Keyword,
    Group(String),
}

impl ParserContext<'_, GroupDefinitionToken> {
    pub fn parse(&mut self) -> Option<Group> {
        self.consume_keyword();
        let name = self.consume_type_name()?;
        let count = self.try_consume_quantifier();
        let mut groups = vec![];

        loop {
            let t = next_or_none!(self, "Expected '|' or ';' after group identifier")?;
            self.tokens.push(t.to_semantic_token(u32::MAX));
            match t.kind() {
                GroupDefinitionToken::Semicolon => return Some(Group::new(name, groups, count)),
                GroupDefinitionToken::Pipe => groups.push(self.parse_ref_group()?),
                GroupDefinitionToken::SyntaxError => self.report_error_message("Syntax error"),
                kind => self.report_error_message(&format!("Expected ';' or '[', got {kind}")),
            }
        }
    }

    fn parse_ref_group(&mut self) -> Option<RefGroup> {
        match self.consume_pipe_identifier()? {
            PipeIdentifier::Group(name) => Some(RefGroup {
                unique: false,
                namespace: None, // TODO namespace
                name,
                count: self.try_consume_quantifier(),
            }),
            PipeIdentifier::Keyword => {
                if let PipeIdentifier::Group(name) = self.consume_pipe_identifier()? {
                    Some(RefGroup {
                        unique: true,
                        namespace: None, // TODO namespace
                        name,
                        count: self.try_consume_quantifier(),
                    })
                } else {
                    self.report_error_message("Expected identifier after `unique`");
                    None
                }
            }
        }
    }

    fn consume_pipe_identifier(&mut self) -> Option<PipeIdentifier> {
        let identifier = peek_or_none!(self)?;
        match identifier.kind() {
            GroupDefinitionToken::Identifier => {
                let t = next_or_none!(self).unwrap();
                let slice = t.slice(self.src);
                if slice == "unique" {
                    self.tokens.push(t.to_semantic_token(KEYWORD_TOKEN));
                    return Some(PipeIdentifier::Keyword);
                }

                self.tokens.push(t.to_semantic_token(TYPE_TOKEN));
                return Some(PipeIdentifier::Group(slice.to_string()));
            }
            GroupDefinitionToken::SyntaxError => self.consume_error("Syntax error"),
            kind => self.consume_error(&format!(
                "Expected 'unique' keyword or group identifier, got {kind}"
            )),
        }

        None
    }

    fn try_consume_quantifier(&mut self) -> Option<Count> {
        let quantifier = peek_or_none!(self)?;
        match quantifier.kind() {
            GroupDefinitionToken::Quantifier(tokens) => {
                next_or_none!(self).unwrap();
                return ParserContext::new(
                    tokens.iter().peekable(),
                    self.diagnostics,
                    self.tokens,
                    self.src,
                )
                .parse();
            }
            GroupDefinitionToken::SyntaxError => self.consume_error("Syntax error"),
            _ => return None,
        }
        None
    }
}

impl ParserContext<'_, QuantifierToken> {
    pub fn parse(&mut self) -> Option<Count> {
        let first = next_or_none!(self)?;
        self.tokens.push(first.to_semantic_token(MACRO_TOKEN));
        match first.kind() {
            QuantifierToken::ZeroOrOne => return Some(Count::Range(0..1)),
            QuantifierToken::ZeroOrMore => return Some(Count::Range(0..u32::MAX)),
            QuantifierToken::OneOrMore => return Some(Count::Range(1..u32::MAX)),
            QuantifierToken::SyntaxError => self.report_error_message("Syntax error"),
            _ => {}
        }

        let first_number = self.consume_range_number()?;

        let range_or_bracket = peek_or_none!(self)?;
        match range_or_bracket.kind() {
            QuantifierToken::RightSquareBracket => {
                let t = next_or_none!(self).unwrap();
                self.tokens.push(t.to_semantic_token(MACRO_TOKEN));
                return Some(Count::Single(first_number));
            }
            QuantifierToken::Range => {
                let t = next_or_none!(self).unwrap();
                self.tokens.push(t.to_semantic_token(MACRO_TOKEN));
            }
            QuantifierToken::SyntaxError => self.consume_error("Syntax error"),
            kind => self.consume_error(&format!("Expected range or '[', got {kind}")),
        }

        let second_number = self.consume_range_number()?;

        let bracket = next_or_none!(self)?;
        self.tokens.push(bracket.to_semantic_token(MACRO_TOKEN));
        match bracket.kind() {
            QuantifierToken::RightSquareBracket => {
                return Some(Count::Range(first_number..second_number))
            }
            QuantifierToken::SyntaxError => self.report_error_message("Syntax error"),
            kind => self.report_error_message(&format!("Expected range or '[', got {kind}")),
        }

        None
    }

    fn consume_range_number(&mut self) -> Option<u32> {
        let number = next_or_none!(self)?;
        self.tokens.push(number.to_semantic_token(MACRO_TOKEN));
        match number.kind() {
            QuantifierToken::Number => {
                let number = number.slice(self.src).parse::<u32>().expect("unreachable");
                return Some(number);
            }
            QuantifierToken::SyntaxError => self.report_error_message("Syntax error"),
            kind => self.report_error_message(&format!("Expected number, got {kind}")),
        }

        None
    }
}
