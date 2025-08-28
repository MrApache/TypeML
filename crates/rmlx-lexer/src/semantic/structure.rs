use lexer_utils::{Token, KEYWORD_TOKEN, OPERATOR_TOKEN, PARAMETER_TOKEN, TYPE_TOKEN};
use std::slice::Iter;
use tower_lsp::lsp_types::SemanticToken;

use crate::{StructToken, TokenDefinition};

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
    Simple(String),          // например Display
    Generic(String, String), // например Option<f32>
}

pub struct ParserContext<'s, T: TokenDefinition> {
    tokens: &'s mut Vec<SemanticToken>,
    iter: Iter<'s, Token<T>>,
    src: &'s str,
}

impl<'s, T: TokenDefinition> ParserContext<'s, T> {
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
        if brace.kind() != &T::left_curly_brace() {
            return Err(format!("Expected `{{`, found '{}'", brace.kind()));
        }
        self.tokens.push(brace.to_semantic_token(u32::MAX));
        Ok(())
    }

    pub fn consume_type_name(&mut self) -> Result<String, String> {
        let name_tok = self.iter.next().ok_or("Unexpected EOF, expected identifier")?;
        if name_tok.kind() != &T::identifier() {
            return Err(format!("Expected identifier, found '{}'", name_tok.kind()));
        }
        self.tokens.push(name_tok.to_semantic_token(TYPE_TOKEN));
        Ok(name_tok.slice(self.src).to_string())
    }

    pub fn consume_colon(&mut self) -> Result<(), String> {
        let colon = self.iter.next().ok_or("Unexpected EOF, expected `:`")?;
        if colon.kind() != &T::colon() {
            return Err(format!("Expected ':', found '{}'", colon.kind()));
        }
        self.tokens.push(colon.to_semantic_token(u32::MAX));
        Ok(())
    }
}

impl<'s> ParserContext<'s, StructToken> {
    pub fn parse(&mut self) -> Result<Struct, String> {
        self.consume_keyword()?;
        let name = self.consume_type_name()?;
        self.consume_left_curve_brace()?;

        let mut fields = Vec::new();
        loop {
            let next = self.iter.next().ok_or("Unexpected EOF in struct body")?;
            match next.kind() {
                // конец структуры
                StructToken::RightCurlyBracket => {
                    self.tokens.push(next.to_semantic_token(u32::MAX));
                    break;
                }

                StructToken::Identifier => {
                    self.tokens.push(next.to_semantic_token(PARAMETER_TOKEN));
                    let field_name = next.slice(self.src).to_string();

                    self.consume_colon()?;

                    let type_name = self.consume_type_name()?;

                    let ty = {
                        let peek = self.iter.clone().next();
                        if let Some(tok) = peek {
                            if tok.kind() == &StructToken::LeftAngleBracket {
                                // съедаем <
                                {
                                    let t = self.iter.next().unwrap();
                                    self.tokens.push(t.to_semantic_token(OPERATOR_TOKEN));
                                }

                                let inner_type_name = self.consume_type_name()?;

                                {
                                    let close =
                                        self.iter.next().ok_or("Unexpected EOF, expected `>`")?;
                                    if close.kind() != &StructToken::RightAngleBracket {
                                        return Err("Expected `>` after generic type".into());
                                    }
                                    self.tokens.push(close.to_semantic_token(OPERATOR_TOKEN));
                                }

                                // Возвращаем Generic из блока
                                Type::Generic(type_name, inner_type_name)
                            } else {
                                // если токен есть, но не '<'
                                Type::Simple(type_name)
                            }
                        } else {
                            // если peek None
                            Type::Simple(type_name)
                        }
                    };

                    fields.push(Field {
                        name: field_name,
                        ty,
                    });

                    // после поля может быть , или }
                    let sep = self.iter.next().ok_or("Unexpected EOF after field")?;
                    self.tokens.push(sep.to_semantic_token(u32::MAX));
                    match sep.kind() {
                        StructToken::Comma => continue,
                        StructToken::RightCurlyBracket => break,
                        _ => return Err("Expected `,` or `}` after field".into()),
                    }
                }
                StructToken::NewLine | StructToken::Whitespace => unreachable!(),
                _ => return Err("Unexpected token in struct body".into()),
            }
        }

        Ok(Struct { name, fields })
    }
}
