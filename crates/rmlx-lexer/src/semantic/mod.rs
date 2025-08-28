mod attribute;
mod enumeration;
mod expression;
mod group;
mod structure;
mod use_statement;

pub use attribute::*;
pub use enumeration::*;
pub use expression::*;
pub use group::*;
pub use structure::*;
pub use use_statement::*;

use crate::{NamedStatement, TokenArrayProvider, TokenDefinition, TokenSimpleTypeProvider};
use lexer_utils::*;
use std::{iter::Peekable, slice::Iter};
use tower_lsp::lsp_types::SemanticToken;

pub struct ParserContext<'s, T: TokenDefinition> {
    tokens: &'s mut Vec<SemanticToken>,
    iter: Peekable<Iter<'s, Token<T>>>,
    src: &'s str,
}

impl<'s, T: TokenDefinition> ParserContext<'s, T> {
    pub fn new(
        tokens: &'s mut Vec<SemanticToken>,
        iter: Peekable<Iter<'s, Token<T>>>,
        src: &'s str,
    ) -> Self {
        Self { tokens, iter, src }
    }

    pub fn consume_keyword(&mut self) -> Result<(), String> {
        let t = self
            .iter
            .next()
            .ok_or(format!("Unexpected EOF, expected `{}`", T::keyword()))?;
        if t.kind() != &T::keyword_token() {
            return Err(format!("Expected `{}` keyword", T::keyword()));
        }
        self.tokens.push(t.to_semantic_token(KEYWORD_TOKEN));
        Ok(())
    }

    pub fn consume_left_curve_brace(&mut self) -> Result<(), String> {
        let brace = self.iter.next().ok_or("Unexpected EOF, expected `{`")?;
        if brace.kind() != &T::left_curly_bracket() {
            return Err(format!("Expected `{{`, found '{}'", brace.kind()));
        }
        self.tokens.push(brace.to_semantic_token(u32::MAX));
        Ok(())
    }

    pub fn consume_colon(&mut self) -> Result<(), String> {
        let colon = self.iter.next().ok_or("Unexpected EOF, expected `:`")?;
        if colon.kind() != &T::colon() {
            return Err(format!("Expected ':', found '{}'", colon.kind()));
        }
        self.tokens.push(colon.to_semantic_token(u32::MAX));
        Ok(())
    }

    pub fn next_or_error(&mut self) -> Result<&Token<T>, &str> {
        self.iter.next().ok_or("Unexpected end of token stream")
    }

    pub fn consume_parameter(&mut self) -> Result<String, String> {
        let next = self
            .iter
            .next()
            .ok_or("Unexpected end of token stream in statement body")?;
        self.tokens.push(next.to_semantic_token(PARAMETER_TOKEN));
        Ok(next.slice(self.src).to_string())
    }
}

impl<'s, T: TokenDefinition + NamedStatement> ParserContext<'s, T> {
    pub fn consume_type_name(&mut self) -> Result<String, String> {
        let name_tok = self
            .iter
            .next()
            .ok_or("Unexpected EOF, expected identifier")?;
        if name_tok.kind() != &T::identifier() {
            return Err(format!("Expected identifier, found '{}'", name_tok.kind()));
        }
        self.tokens.push(name_tok.to_semantic_token(TYPE_TOKEN));
        Ok(name_tok.slice(self.src).to_string())
    }
}

impl<'s, T: TokenDefinition + TokenSimpleTypeProvider> ParserContext<'s, T> {
    pub fn consume_typed_field(&mut self) -> Result<Field, String> {
        let field_name = self.consume_parameter()?;
        self.consume_colon()?;
        let ty = self.consume_simple_or_generic_type()?;

        Ok(Field {
            name: field_name,
            ty,
        })
    }

    fn consume_simple_or_generic_type(&mut self) -> Result<Type, String> {
        let type_name = self.consume_type_name()?;
        let peek = self.iter.clone().next();
        if let Some(tok) = peek {
            if tok.kind() == &T::left_angle_bracket() {
                // съедаем <
                {
                    let t = self.iter.next().unwrap();
                    self.tokens.push(t.to_semantic_token(OPERATOR_TOKEN));
                }

                let inner_type_name = self.consume_type_name()?;

                {
                    let close = self.iter.next().ok_or("Unexpected EOF, expected `>`")?;
                    if close.kind() != &T::right_angle_bracket() {
                        return Err("Expected `>` after generic type".into());
                    }
                    self.tokens.push(close.to_semantic_token(OPERATOR_TOKEN));
                }

                // Возвращаем Generic из блока
                Ok(Type::Generic(type_name, inner_type_name))
            } else {
                // если токен есть, но не '<'
                Ok(Type::Simple(type_name))
            }
        } else {
            // если peek None
            Ok(Type::Simple(type_name))
        }
    }
}

impl<'s, T: TokenDefinition + TokenArrayProvider> ParserContext<'s, T> {
    pub fn consume_array(&mut self) -> Result<Vec<String>, String> {
        let lsb_token = self.iter.next().unwrap();
        self.tokens.push(lsb_token.to_semantic_token(u32::MAX));

        let mut arr = Vec::new();
        loop {
            let type_token = self.iter.next().ok_or("Unexpected end of token stream")?;

            if type_token.kind() == &T::right_square_bracket() {
                self.tokens.push(type_token.to_semantic_token(u32::MAX));
                break;
            } else if type_token.kind() == &T::identifier() {
                self.tokens.push(type_token.to_semantic_token(TYPE_TOKEN));
                arr.push(type_token.slice(self.src).to_string());
            } else {
                return Err("Expected identifier inside array".into());
            }

            // ',' or ']'
            if let Some(tok) = self.iter.peek_mut() {
                if tok.kind() == &T::comma() {
                    let tok = self.iter.next().unwrap();
                    self.tokens.push(tok.to_semantic_token(u32::MAX));
                } else if tok.kind() == &T::right_square_bracket() {
                    continue;
                } else {
                    return Err("Expected ',' or ']' in array".into());
                }
            }
        }

        Ok(arr)
    }
}

impl<'s, T: TokenDefinition + TokenSimpleTypeProvider + TokenArrayProvider> ParserContext<'s, T> {
    pub fn consume_advanced_typed_field(&mut self) -> Result<Field, String> {
        let field_name = self.consume_parameter()?;
        self.consume_colon()?;

        let kind = self.iter.peek().ok_or("Unexpected end of token stream")?;
        if kind.kind() == &T::left_square_bracket() {
            Ok(Field {
                name: field_name,
                ty: Type::Array(self.consume_array()?),
            })
        } else if kind.kind() == &T::left_curly_bracket() {
            Ok(Field {
                name: field_name,
                ty: self.consume_block()?,
            })
        }
        else {
            Ok(Field {
                name: field_name,
                ty: self.consume_simple_or_generic_type()?
            })
        }
    }

    pub fn consume_block(&mut self) -> Result<Type, String> {
        let lcb_token = self.iter.next().unwrap();
        self.tokens.push(lcb_token.to_semantic_token(u32::MAX));

        let mut block = Vec::new();
        loop {
            let token = self.iter.peek().ok_or("Unexpected end of token stream")?;
            if token.kind() == &T::right_curly_bracket() {
                let tok = self.iter.next().unwrap();
                self.tokens.push(tok.to_semantic_token(u32::MAX));
                break;
            } else if token.kind() == &T::comma() {
                let tok = self.iter.next().unwrap();
                self.tokens.push(tok.to_semantic_token(u32::MAX));
            } else if token.kind() == &T::identifier() {
                block.push(self.consume_typed_field()?);
            } else {
                return Err("Expected ',' or '}' in block".into());
            }
        }

        Ok(Type::Block(block))
    }
}
