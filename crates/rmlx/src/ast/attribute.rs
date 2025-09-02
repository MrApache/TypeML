use crate::{next_or_none, AttributeToken, ContentToken, ParserContext};
use lexer_core::{MACRO_TOKEN, STRING_TOKEN};

#[derive(Debug)]
pub struct Attribute {
    name: String,
    content: Option<String>,
}

impl ParserContext<'_, AttributeToken> {
    pub fn parse(&mut self) -> Option<Vec<Attribute>> {
        self.consume_keyword_with_token_type(MACRO_TOKEN);
        self.consume_left_square_bracket()?;

        let mut attrs = Vec::new();

        loop {
            let name_token = next_or_none!(self)?;
            match name_token.kind() {
                AttributeToken::Comma => {
                    self.tokens.push(name_token.to_semantic_token(u32::MAX));
                    continue;
                }
                AttributeToken::RightSquareBracket => {
                    self.tokens.push(name_token.to_semantic_token(MACRO_TOKEN));
                    break;
                }
                AttributeToken::Identifier => {
                    self.tokens.push(name_token.to_semantic_token(MACRO_TOKEN));
                }
                AttributeToken::SyntaxError => self.report_error(name_token, "Syntax error"),
                kind => self.report_error(name_token, &format!("Expected identifier, got {kind}")),
            }

            let name = name_token.slice(self.src).to_string();

            let next = next_or_none!(self, "Expected content or , or ]")?;
            match next.kind() {
                AttributeToken::Content(inner_tokens) => {
                    let content = ParserContext::new(
                        inner_tokens.iter().peekable(),
                        self.diagnostics,
                        self.tokens,
                        self.src,
                    )
                    .parse();
                    attrs.push(Attribute { name, content });
                }
                AttributeToken::Comma => self.tokens.push(next.to_semantic_token(u32::MAX)),
                AttributeToken::RightSquareBracket => {
                    self.tokens.push(next.to_semantic_token(MACRO_TOKEN));
                    break;
                }
                AttributeToken::SyntaxError => self.report_error(next, "Syntax error"),
                kind => {
                    self.report_error(next, &format!("Unexpected token after identifier: {kind}"));
                }
            }
        }

        Some(attrs)
    }

    fn consume_left_square_bracket(&mut self) -> Option<()> {
        let left_square = next_or_none!(self)?;
        self.tokens.push(left_square.to_semantic_token(MACRO_TOKEN));
        match left_square.kind() {
            AttributeToken::LeftSquareBracket => return Some(()),
            AttributeToken::SyntaxError => self.report_error(left_square, "Syntax error"),
            kind => self.report_error(left_square, &format!("Expected '[', got {kind}")),
        }

        None
    }
}

impl ParserContext<'_, ContentToken> {
    pub fn parse(&mut self) -> Option<String> {
        self.consume_keyword_with_token_type(u32::MAX);

        let t = next_or_none!(self, "Expected String or Value")?;
        let result_text = t.slice(self.src);
        let result_text = match t.kind() {
            ContentToken::String => {
                self.tokens.push(t.to_semantic_token(STRING_TOKEN));
                t.slice(self.src).trim_matches('"')
            }
            ContentToken::Value => {
                self.tokens.push(t.to_semantic_token(STRING_TOKEN));
                result_text
            }
            ContentToken::SyntaxError => {
                self.report_error(t, "Syntax error");
                ""
            }
            _ => {
                self.report_error(t, "Expected value");
                ""
            }
        };

        self.consume_right_par()?;

        Some(result_text.to_string())
    }

    fn consume_right_par(&mut self) -> Option<()> {
        let right_par = next_or_none!(self, "Expected ')'")?;
        match right_par.kind() {
            ContentToken::RightParenthesis => {
                self.tokens.push(right_par.to_semantic_token(u32::MAX));
                return Some(());
            }
            ContentToken::SyntaxError => self.report_error(right_par, "Syntax error"),
            kind => self.report_error(right_par, &format!("Expected ')', got {kind}")),
        }

        None
    }
}
