use crate::{next_or_none, AttributeToken, ContentToken, ParserContext};
use lexer_utils::{MACRO_TOKEN, STRING_TOKEN};

#[derive(Debug)]
pub struct Attribute {
    name: String,
    content: Option<String>,
}

impl<'s> ParserContext<'s, AttributeToken> {
    pub fn parse(&mut self) -> Option<Vec<Attribute>> {
        self.consume_keyword_with_token_type(MACRO_TOKEN);

        let mut attrs = Vec::new();

        {
            // consume `[`
            let t = next_or_none!(self)?;
            self.tokens.push(t.to_semantic_token(MACRO_TOKEN));
            match t.kind() {
                AttributeToken::LeftSquareBracket => {} //continue
                AttributeToken::SyntaxError => self.report_error(t, "Syntax error"),
                kind => self.report_error(t, &format!("Expected '[', got {kind}")),
            }
        }

        loop {
            // читаем идентификатор
            let name_token = next_or_none!(self)?;
            match name_token.kind() {
                AttributeToken::Comma => {
                    self.tokens.push(name_token.to_semantic_token(u32::MAX));
                    continue;
                }
                AttributeToken::RightSquareBracket => {
                    self.tokens.push(name_token.to_semantic_token(u32::MAX));
                    break;
                }
                AttributeToken::Identifier => {
                    self.tokens.push(name_token.to_semantic_token(MACRO_TOKEN))
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
                AttributeToken::Comma => {
                    self.tokens.push(next.to_semantic_token(u32::MAX));
                }
                AttributeToken::SyntaxError => self.report_error(next, "Syntax error"),
                kind => {
                    self.report_error(next, &format!("Unexpected token after identifier: {kind}"))
                }
            }
        }

        Some(attrs)
    }
}

impl<'s> ParserContext<'s, ContentToken> {
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

        let t = next_or_none!(self, "Expected ')'")?;
        match t.kind() {
            ContentToken::RightParenthesis => self.tokens.push(t.to_semantic_token(u32::MAX)),
            ContentToken::SyntaxError => self.report_error(t, "Syntax error"),
            kind => self.report_error(t, &format!("Expected ')', got {kind}")),
        }

        Some(result_text.to_string())
    }
}
