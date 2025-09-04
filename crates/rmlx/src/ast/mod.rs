mod attribute;
mod directive;
mod enumeration;
mod expression;
mod group;
mod r#type;

pub use attribute::*;
pub use directive::*;
pub use enumeration::*;
pub use expression::*;
pub use group::*;
pub use r#type::*;

use crate::{
    NamedStatement, RmlxTokenStream, StatementTokens, TokenArrayProvider, TokenBodyStatement, TokenSimpleTypeProvider,
};
use lexer_core::{Token, KEYWORD_TOKEN, TYPE_TOKEN};
use std::{iter::Peekable, slice::Iter};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Range};

#[macro_export]
macro_rules! next_or_none {
    ($self:expr) => {
        next_or_none!($self, "Unexpected end of token stream")
    };
    ($self:expr, $msg:expr) => {{
        use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};
        let token = $self.iter.next();
        if token.is_none() {
            $self.diagnostics.push(Diagnostic {
                range: $self.previous_token_range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: $msg.into(),
                ..Default::default()
            });
        } else {
            $self.previous_token_range = token.unwrap().range();
        }
        token
    }};
}

#[macro_export]
macro_rules! peek_or_none {
    ($self:expr) => {
        peek_or_none!($self, "Unexpected end of token stream")
    };
    ($self:expr, $msg:expr) => {{
        use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};
        let token = $self.iter.peek();
        if token.is_none() {
            $self.diagnostics.push(Diagnostic {
                range: $self.previous_token_range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: $msg.into(),
                ..Default::default()
            });
        }
        token
    }};
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    name: String,
    ty: Type,
}

impl Field {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Simple(String),
    Generic(String, String),
    Array(Vec<String>),
    Block(Vec<Field>),
}

pub struct ParserContext<'s, T> {
    iter: Peekable<Iter<'s, Token<T>>>,
    diagnostics: &'s mut Vec<Diagnostic>,
    src: &'s str,

    statement_range: Range,
    previous_token_range: Range,
}

impl<'s, T> ParserContext<'s, T> {
    pub fn new(iter: Peekable<Iter<'s, Token<T>>>, diagnostics: &'s mut Vec<Diagnostic>, src: &'s str) -> Self {
        Self {
            iter,
            diagnostics,
            src,
            statement_range: Range::default(),
            previous_token_range: Range::default(),
        }
    }

    pub fn consume_keyword(&mut self) -> &str {
        let keyword = self.iter.next().unwrap();
        self.statement_range = keyword.range();
        self.previous_token_range = self.statement_range;
        keyword.slice(self.src)
    }

    pub fn consume_parameter(&mut self) -> Option<String> {
        let parameter = next_or_none!(self)?;
        Some(parameter.slice(self.src).to_string())
    }

    fn consume_error(&mut self, message: &str) {
        next_or_none!(self).unwrap();
        self.report_error(message);
    }

    fn report_error(&mut self, message: &str) {
        self.diagnostics.push(Diagnostic {
            range: self.previous_token_range,
            severity: Some(DiagnosticSeverity::ERROR),
            message: message.into(),
            ..Default::default()
        });
    }
}

impl<T: StatementTokens + TokenBodyStatement> ParserContext<'_, T> {
    pub fn consume_left_curve_brace(&mut self) -> Option<()> {
        let brace = next_or_none!(self, "Unexpected end of token stream, expected '{'")?;
        if brace.kind() == &T::left_curly_bracket() {
        } else {
            self.report_error(&format!("Expected `{{`, found '{}'", brace.kind()));
        }
        Some(())
    }
}

impl<T: StatementTokens + NamedStatement> ParserContext<'_, T> {
    pub fn consume_type_name(&mut self) -> Option<String> {
        let name = next_or_none!(self, "Unexpected end of token stream, expected identifier")?;
        if name.kind() == &T::identifier() {
        } else {
            self.report_error(&format!("Expected identifier, found '{}'", name.kind()));
        }
        Some(name.slice(self.src).to_string())
    }
}

impl<T: StatementTokens + TokenSimpleTypeProvider> ParserContext<'_, T> {
    pub fn consume_colon(&mut self) -> Option<()> {
        let colon = next_or_none!(self, "Unexpected end of token stream, expected ':'")?;
        if colon.kind() == &T::colon() {
        } else {
            self.report_error(&format!("Expected ':', found '{}'", colon.kind()));
        }
        Some(())
    }

    pub fn consume_typed_field(&mut self) -> Option<Field> {
        let name = self.consume_parameter()?;
        self.consume_colon()?;
        let ty = self.consume_simple_or_generic_type()?;

        Some(Field { name, ty })
    }

    fn consume_simple_or_generic_type(&mut self) -> Option<Type> {
        let type_name = self.consume_type_name()?;

        let peek = self.iter.peek();
        if let Some(token) = peek {
            if token.kind() == &T::left_angle_bracket() {
                // съедаем <
                {
                    next_or_none!(self).unwrap();
                }

                let inner_type_name = self.consume_type_name()?;

                {
                    let close = next_or_none!(self, "Unexpected end of token stream, expected '>'")?;
                    if close.kind() == &T::right_angle_bracket() {
                    } else {
                        self.report_error("Expected '>' after generic type");
                    }
                }

                Some(Type::Generic(type_name, inner_type_name))
            } else {
                Some(Type::Simple(type_name))
            }
        } else {
            Some(Type::Simple(type_name))
        }
    }
}

impl<T: StatementTokens + TokenArrayProvider> ParserContext<'_, T> {
    pub fn consume_array(&mut self) -> Option<Vec<String>> {
        let lsb_token = next_or_none!(self).expect("Call this method after peeking");

        let mut arr = Vec::new();
        loop {
            let type_token = next_or_none!(self)?;

            if type_token.kind() == &T::right_square_bracket() {
                break;
            } else if type_token.kind() == &T::identifier() {
                arr.push(type_token.slice(self.src).to_string());
            } else {
                self.report_error("Expected identifier inside array");
            }

            if let Some(tok) = self.iter.peek_mut() {
                if tok.kind() == &T::comma() {
                    let tok = next_or_none!(self).unwrap();
                } else if tok.kind() != &T::right_square_bracket() {
                    self.consume_error("Expected ',' or ']' in array");
                }
            }
        }

        Some(arr)
    }
}

impl<T> ParserContext<'_, T>
where
    T: StatementTokens + TokenSimpleTypeProvider + TokenArrayProvider + TokenBodyStatement,
{
    pub fn consume_advanced_typed_field(&mut self) -> Option<Field> {
        let name = self.consume_parameter()?;
        self.consume_colon()?;

        let kind = peek_or_none!(self)?;
        let ty = if kind.kind() == &T::left_square_bracket() {
            Type::Array(self.consume_array()?)
        } else if kind.kind() == &T::left_curly_bracket() {
            self.consume_block()?
        } else {
            self.consume_simple_or_generic_type()?
        };

        Some(Field { name, ty })
    }

    pub fn consume_block(&mut self) -> Option<Type> {
        let lcb_token = next_or_none!(self).expect("Call this method after peeking");

        let mut block = Vec::new();
        loop {
            let token = peek_or_none!(self)?;
            if token.kind() == &T::right_curly_bracket() {
                let tok = next_or_none!(self).unwrap();
                break;
            } else if token.kind() == &T::comma() {
                let tok = next_or_none!(self).unwrap();
            } else if token.kind() == &T::identifier() {
                block.push(self.consume_typed_field()?);
            } else {
                self.consume_error("Expected ',' or '}' in block");
            }
        }

        Some(Type::Block(block))
    }
}

#[derive(Default, Debug)]
pub struct SchemaAst {
    directives: Vec<Directive>,
    enums: Vec<Enum>,
    groups: Vec<Group>,
    extendable_groups: Vec<Group>,
    types: Vec<TypeDefinition>,
    expressions: Vec<Expression>,

    pub diagnostics: Vec<Diagnostic>,
}

impl SchemaAst {
    #[must_use]
    pub fn directives(&self) -> &[Directive] {
        &self.directives
    }

    #[must_use]
    pub fn enumerations(&self) -> &[Enum] {
        &self.enums
    }

    #[must_use]
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    #[must_use]
    pub fn extendable_groups(&self) -> &[Group] {
        &self.extendable_groups
    }

    #[must_use]
    pub fn types(&self) -> &[TypeDefinition] {
        &self.types
    }

    #[must_use]
    pub fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
}

impl SchemaAst {
    #[must_use]
    pub fn new(content: &str) -> Self {
        let mut schema = SchemaAst::default();
        let mut stream = RmlxTokenStream::new(content);
        let mut attributes = vec![];

        while let Some(token) = stream.next_token() {
            match token {
                crate::SchemaStatement::Attribute(tokens) => {
                    let attrs = ParserContext::new(tokens.iter().peekable(), &mut schema.diagnostics, content).parse();

                    if let Some(attrs) = attrs {
                        attributes = attrs;
                    }
                }
                crate::SchemaStatement::Group(tokens) => {
                    let group = ParserContext::new(tokens.iter().peekable(), &mut schema.diagnostics, content).parse();

                    if let Some(mut group) = group {
                        group.set_attributes(std::mem::take(&mut attributes));
                        schema.groups.push(group);
                    }
                }
                crate::SchemaStatement::ExtendGroup(tokens) => {
                    let group = ParserContext::new(tokens.iter().peekable(), &mut schema.diagnostics, content).parse();

                    if let Some(mut group) = group {
                        group.set_attributes(std::mem::take(&mut attributes));
                        schema.extendable_groups.push(group);
                    }
                }
                crate::SchemaStatement::Expression(tokens) => {
                    let expression =
                        ParserContext::new(tokens.iter().peekable(), &mut schema.diagnostics, content).parse();

                    if let Some(expression) = expression {
                        schema.expressions.push(expression);
                    }
                }
                crate::SchemaStatement::Enum(tokens) => {
                    let enumeration =
                        ParserContext::new(tokens.iter().peekable(), &mut schema.diagnostics, content).parse();

                    if let Some(mut enumeration) = enumeration {
                        enumeration.set_attributes(std::mem::take(&mut attributes));
                        schema.enums.push(enumeration);
                    }
                }
                crate::SchemaStatement::Directive(tokens) => {
                    let directive =
                        ParserContext::new(tokens.iter().peekable(), &mut schema.diagnostics, content).parse();

                    if let Some(directive) = directive {
                        schema.directives.push(directive);
                    }
                }
                crate::SchemaStatement::Type(tokens) => {
                    let r#type = ParserContext::new(tokens.iter().peekable(), &mut schema.diagnostics, content).parse();

                    if let Some(mut r#type) = r#type {
                        r#type.attributes = std::mem::take(&mut attributes);
                        schema.types.push(r#type);
                    }
                }
                crate::SchemaStatement::SyntaxError(token) => {
                    schema.diagnostics.push(Diagnostic {
                        range: token.range(),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: "Syntax error".to_string(),
                        ..Default::default()
                    });
                }
                crate::SchemaStatement::Comment(_)
                | crate::SchemaStatement::NewLine
                | crate::SchemaStatement::Whitespace => {}
            }
        }

        schema
    }
}
