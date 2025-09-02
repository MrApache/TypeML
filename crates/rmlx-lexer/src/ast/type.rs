use crate::{next_or_none, peek_or_none, Attribute, Field, ParserContext, TypeDefinitionToken};
use lexer_utils::{OPERATOR_TOKEN, TYPE_TOKEN};

#[derive(Debug)]
pub struct RefType {
    namespace: Option<String>,
    name: String,
}

#[derive(Debug)]
pub struct Type {
    pub keyword: String,
    pub name: String,
    pub fields: Vec<Field>,
    pub bind: Option<RefType>,
    pub attributes: Vec<Attribute>,
}

impl ParserContext<'_, TypeDefinitionToken> {
    pub fn parse(&mut self) -> Option<Type> {
        let keyword = self.consume_keyword().to_string();
        let name = self.consume_type_name()?;
        let bind = self.try_parse_binding();
        let mut fields = Vec::new();

        let next = peek_or_none!(self)?;
        match next.kind() {
            TypeDefinitionToken::Semicolon => {
                let next = next_or_none!(self).unwrap();
                self.tokens.push(next.to_semantic_token(u32::MAX));
            }
            TypeDefinitionToken::LeftCurlyBracket => {
                self.consume_left_curve_brace()?;

                loop {
                    let next = peek_or_none!(
                        self,
                        "Unexpected end of token stream in type definition body"
                    )?;
                    match next.kind() {
                        TypeDefinitionToken::RightCurlyBracket | TypeDefinitionToken::Semicolon => {
                            let next = next_or_none!(self).unwrap();
                            self.tokens.push(next.to_semantic_token(u32::MAX));
                            break;
                        }

                        TypeDefinitionToken::Identifier => {
                            fields.push(self.consume_typed_field()?);

                            let sep =
                                next_or_none!(self, "Unexpected end of token stream after field")?;
                            self.tokens.push(sep.to_semantic_token(u32::MAX));
                            match sep.kind() {
                                TypeDefinitionToken::Comma => {}, // continue
                                TypeDefinitionToken::RightCurlyBracket => break,
                                TypeDefinitionToken::SyntaxError => {
                                    self.report_error(sep, "Syntax error");
                                }
                                _ => self.report_error(sep, "Expected ',' or '}' after field"),
                            }
                        }
                        TypeDefinitionToken::NewLine | TypeDefinitionToken::Whitespace => {
                            unreachable!()
                        }
                        TypeDefinitionToken::SyntaxError => self.consume_error("Syntax error"),
                        _ => self.consume_error("Unexpected token in type definition body"),
                    }
                }
            }
            TypeDefinitionToken::SyntaxError => self.consume_error("Syntax error"),
            _ => self.consume_error("Unexpected token in type definition body"),
        }

        Some(Type {
            keyword,
            name,
            fields,
            bind,
            attributes: vec![],
        })
    }

    fn try_parse_binding(&mut self) -> Option<RefType> {
        let dash = peek_or_none!(self)?;
        match dash.kind() {
            TypeDefinitionToken::Dash => {
                let t = next_or_none!(self).unwrap();
                self.tokens.push(t.to_semantic_token(OPERATOR_TOKEN));
            }
            TypeDefinitionToken::SyntaxError => {
                self.consume_error("Syntax error");
                return None;
            }
            _ => return None,
        }

        let right_angle_bracket = next_or_none!(self)?;
        match right_angle_bracket.kind() {
            TypeDefinitionToken::RightAngleBracket => {
                self.tokens
                    .push(right_angle_bracket.to_semantic_token(OPERATOR_TOKEN));
            }
            TypeDefinitionToken::SyntaxError => {
                self.report_error(right_angle_bracket, "Syntax error");
                return None;
            }
            kind => {
                self.report_error(right_angle_bracket, &format!("Expected '>', got {kind}"));
                return None;
            }
        }

        let identifier = next_or_none!(self)?;
        match identifier.kind() {
            TypeDefinitionToken::Identifier => {
                self.tokens.push(identifier.to_semantic_token(TYPE_TOKEN));
                Some(RefType {
                    namespace: None, //TODO namespace
                    name: identifier.slice(self.src).to_string(),
                })
            }
            TypeDefinitionToken::SyntaxError => {
                self.report_error(identifier, "Syntax error");
                None
            }
            kind => {
                self.report_error(identifier, &format!("Expected identifier, got {kind}"));
                None
            }
        }
    }
}
