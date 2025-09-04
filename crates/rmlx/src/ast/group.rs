use crate::{
    ast::{Attribute, ParserContext},
    next_or_none, peek_or_none, GroupDefinitionToken, QuantifierToken,
};
use std::ops::Range;

#[derive(Debug, PartialEq)]
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
            match t.kind() {
                GroupDefinitionToken::Semicolon => return Some(Group::new(name, groups, count)),
                GroupDefinitionToken::Pipe => groups.push(self.parse_ref_group()?),
                GroupDefinitionToken::SyntaxError => self.report_error("Syntax error"),
                kind => self.report_error(&format!("Expected ';' or '[', got {kind}")),
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
                    self.report_error("Expected identifier after `unique`");
                    None
                }
            }
        }
    }

    fn consume_pipe_identifier(&mut self) -> Option<PipeIdentifier> {
        let identifier = next_or_none!(self)?;
        match identifier.kind() {
            GroupDefinitionToken::Identifier => {
                let slice = identifier.slice(self.src);
                if slice == "unique" {
                    return Some(PipeIdentifier::Keyword);
                }

                return Some(PipeIdentifier::Group(slice.to_string()));
            }
            GroupDefinitionToken::SyntaxError => self.report_error("Syntax error"),
            kind => self.report_error(&format!("Expected 'unique' keyword or group identifier, got {kind}")),
        }

        None
    }

    fn try_consume_quantifier(&mut self) -> Option<Count> {
        let quantifier = peek_or_none!(self)?;
        match quantifier.kind() {
            GroupDefinitionToken::Quantifier(tokens) => {
                next_or_none!(self).unwrap();
                return ParserContext::new(tokens.iter().peekable(), self.diagnostics, self.src).parse();
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
        match first.kind() {
            QuantifierToken::ZeroOrOne => return Some(Count::Range(0..1)),
            QuantifierToken::ZeroOrMore => return Some(Count::Range(0..u32::MAX)),
            QuantifierToken::OneOrMore => return Some(Count::Range(1..u32::MAX)),
            QuantifierToken::SyntaxError => self.report_error("Syntax error"),
            _ => {}
        }

        let first_number = self.consume_range_number()?;

        let range_or_bracket = next_or_none!(self)?;
        match range_or_bracket.kind() {
            QuantifierToken::Range => {} // continue
            QuantifierToken::RightSquareBracket => return Some(Count::Single(first_number)),
            QuantifierToken::SyntaxError => self.report_error("Syntax error"),
            kind => self.report_error(&format!("Expected range or '[', got {kind}")),
        }

        let second_number = self.consume_range_number()?;

        let bracket = next_or_none!(self)?;
        match bracket.kind() {
            QuantifierToken::RightSquareBracket => return Some(Count::Range(first_number..second_number)),
            QuantifierToken::SyntaxError => self.report_error("Syntax error"),
            kind => self.report_error(&format!("Expected range or '[', got {kind}")),
        }

        None
    }

    fn consume_range_number(&mut self) -> Option<u32> {
        let number = next_or_none!(self)?;
        match number.kind() {
            QuantifierToken::Number => return Some(number.slice(self.src).parse::<u32>().expect("unreachable")),
            QuantifierToken::SyntaxError => self.report_error("Syntax error"),
            kind => self.report_error(&format!("Expected number, got {kind}")),
        }

        None
    }
}
