mod attribute;
mod element;
mod enumeration;
mod expression;
mod group;
mod structure;
mod use_statement;

pub use attribute::*;
pub use element::*;
pub use enumeration::*;
pub use expression::*;
pub use group::*;
pub use structure::*;
pub use use_statement::*;

use crate::{
    NamedStatement, TokenArrayProvider, TokenBodyStatement, TokenDefinition,
    TokenSimpleTypeProvider,
};
use lexer_utils::*;
use std::{iter::Peekable, slice::Iter};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Range, SemanticToken};

#[macro_export]
macro_rules! next_or_none {
    ($self:expr) => {
        next_or_none!($self, "Unexpected end of token stream")
    };
    ($self:expr, $msg:expr) => {{
        use tower_lsp::lsp_types::{DiagnosticSeverity, Diagnostic};
        let token = $self.iter.next();
        if token.is_none() {
            $self.diagnostics.push(Diagnostic {
                range: $self.previous_token_range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: $msg.into(),
                ..Default::default()
            });
        }
        else {
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
        use tower_lsp::lsp_types::{DiagnosticSeverity, Diagnostic};
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


pub struct ParserContext<'s, T: TokenDefinition> {
    iter: Peekable<Iter<'s, Token<T>>>,
    diagnostics: &'s mut Vec<Diagnostic>,
    tokens: &'s mut Vec<SemanticToken>,
    src: &'s str,

    statement_range: Range,
    previous_token_range: Range,
}

impl<'s, T: TokenDefinition> ParserContext<'s, T> {
    pub fn new(
        iter: Peekable<Iter<'s, Token<T>>>,
        diagnostics: &'s mut Vec<Diagnostic>,
        tokens: &'s mut Vec<SemanticToken>,
        src: &'s str,
    ) -> Self {
        Self {
            iter,
            diagnostics,
            tokens,
            src,
            statement_range: Default::default(),
            previous_token_range: Default::default(),
        }
    }

    pub fn consume_keyword(&mut self) {
        self.consume_keyword_with_token_type(KEYWORD_TOKEN);
    }

    pub fn consume_keyword_with_token_type(&mut self, token_type: u32) {
        let keyword = self.iter.next().unwrap();
        self.tokens.push(keyword.to_semantic_token(token_type));
        self.statement_range = keyword.range();
        self.previous_token_range = self.statement_range;
    }

    pub fn consume_parameter(&mut self) -> Option<String> {
        let parameter = next_or_none!(self)?;
        self.tokens.push(parameter.to_semantic_token(PARAMETER_TOKEN));
        Some(parameter.slice(self.src).to_string())
    }

    fn create_error_message(&mut self, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic {
            range: self.previous_token_range,
            severity: Some(DiagnosticSeverity::ERROR),
            message: message.into(),
            ..Default::default()
        });
    }

    pub fn transform<U: TokenDefinition>(self, iter: Peekable<Iter<'s, Token<U>>>) -> ParserContext<'s, U> {
        ParserContext::<'s, U> {
            iter,
            diagnostics: self.diagnostics,
            tokens: self.tokens,
            src: self.src,
            statement_range: self.statement_range,
            previous_token_range: self.previous_token_range,
        }
    }
}

impl<'s, T: TokenDefinition + TokenBodyStatement> ParserContext<'s, T> {
    pub fn consume_left_curve_brace(&mut self) -> Option<()> {
        let brace = next_or_none!(self, "Unexpected end of token stream, expected '{'")?;
        if brace.kind() != &T::left_curly_bracket() {
            self.create_error_message(format!("Expected `{{`, found '{}'", brace.kind()));
            return None;
        }
        self.tokens.push(brace.to_semantic_token(u32::MAX));
        Some(())
    }
}

impl<'s, T: TokenDefinition + NamedStatement> ParserContext<'s, T> {
    pub fn consume_type_name(&mut self) -> Option<String> {
        let name = next_or_none!(self, "Unexpected end of token stream, expected identifier")?;
        if name.kind() != &T::identifier() {
            self.create_error_message(format!("Expected identifier, found '{}'", name.kind()));
            return None;
        }
        self.tokens.push(name.to_semantic_token(TYPE_TOKEN));
        Some(name.slice(self.src).to_string())
    }
}

impl<'s, T: TokenDefinition + TokenSimpleTypeProvider> ParserContext<'s, T> {
    pub fn consume_colon(&mut self) -> Option<()> {
        let colon = next_or_none!(self, "Unexpected end of token stream, expected ':'")?;
        if colon.kind() != &T::colon() {
            self.create_error_message(format!("Expected ':', found '{}'", colon.kind()));
            return None;
        }
        self.tokens.push(colon.to_semantic_token(u32::MAX));
        Some(())
    }

    pub fn consume_typed_field(&mut self) -> Option<Field> {
        let name = self.consume_parameter()?;
        self.consume_colon()?;
        let ty = self.consume_simple_or_generic_type()?;

        Some(Field {
            name,
            ty,
        })
    }

    fn consume_simple_or_generic_type(&mut self) -> Option<Type> {
        let type_name = self.consume_type_name()?;

        let peek = self.iter.peek();
        if let Some(token) = peek {
            if token.kind() == &T::left_angle_bracket() {
                // съедаем <
                {
                    let t = next_or_none!(self).expect("Unreachable");
                    self.tokens.push(t.to_semantic_token(OPERATOR_TOKEN));
                }

                let inner_type_name = self.consume_type_name()?;

                {
                    let close = next_or_none!(self, "Unexpected end of token stream, expected '>'")?;
                    if close.kind() != &T::right_angle_bracket() {
                        self.create_error_message("Expected '>' after generic type");
                        return None;
                    }
                    self.tokens.push(close.to_semantic_token(OPERATOR_TOKEN));
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

impl<'s, T: TokenDefinition + TokenArrayProvider> ParserContext<'s, T> {
    pub fn consume_array(&mut self) -> Option<Vec<String>> {
        let lsb_token = next_or_none!(self).expect("Call this method after peeking");
        self.tokens.push(lsb_token.to_semantic_token(u32::MAX));

        let mut arr = Vec::new();
        loop {
            let type_token = next_or_none!(self)?;

            if type_token.kind() == &T::right_square_bracket() {
                self.tokens.push(type_token.to_semantic_token(u32::MAX));
                break;
            } else if type_token.kind() == &T::identifier() {
                self.tokens.push(type_token.to_semantic_token(TYPE_TOKEN));
                arr.push(type_token.slice(self.src).to_string());
            } else {
                self.create_error_message("Expected identifier inside array");
                return None;
            }

            if let Some(tok) = self.iter.peek_mut() {
                if tok.kind() == &T::comma() {
                    let tok = next_or_none!(self).unwrap();
                    self.tokens.push(tok.to_semantic_token(u32::MAX));
                } else if tok.kind() == &T::right_square_bracket() {
                    continue;
                } else {
                    self.create_error_message("Expected ',' or ']' in array");
                    return None;
                }
            }
        }

        Some(arr)
    }
}

impl<'s, T: TokenDefinition + TokenSimpleTypeProvider + TokenArrayProvider + TokenBodyStatement> ParserContext<'s, T> {
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

        Some(Field {
            name,
            ty,
        })
    }

    pub fn consume_block(&mut self) -> Option<Type> {
        let lcb_token = next_or_none!(self).expect("Call this method after peeking");
        self.tokens.push(lcb_token.to_semantic_token(u32::MAX));

        let mut block = Vec::new();
        loop {
            let token = peek_or_none!(self)?;
            if token.kind() == &T::right_curly_bracket() {
                let tok = next_or_none!(self).unwrap();
                self.tokens.push(tok.to_semantic_token(u32::MAX));
                break;
            } else if token.kind() == &T::comma() {
                let tok = next_or_none!(self).unwrap();
                self.tokens.push(tok.to_semantic_token(u32::MAX));
            } else if token.kind() == &T::identifier() {
                block.push(self.consume_typed_field()?);
            } else {
                self.create_error_message("Expected ',' or '}' in block");
                return None;
            }
        }

        Some(Type::Block(block))
    }
}
