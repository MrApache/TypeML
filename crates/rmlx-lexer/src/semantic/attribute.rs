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
            // читаем `[`
            let t = next_or_none!(self)?;
            self.tokens.push(t.to_semantic_token(MACRO_TOKEN));
            if t.kind() != &AttributeToken::LeftSquareBracket {
                self.create_error_message(format!("Expected '[', got {}", t.kind()));
                return None;
            }
        }

        loop {
            // читаем идентификатор
            let t = next_or_none!(self)?;
            if t.kind() == &AttributeToken::Comma {
                self.tokens.push(t.to_semantic_token(u32::MAX));
                continue;
            }
            else if t.kind() == &AttributeToken::RightSquareBracket {
                self.tokens.push(t.to_semantic_token(u32::MAX));
                break;
            }
            else if t.kind() != &AttributeToken::Identifier {
                self.tokens.push(t.to_semantic_token(u32::MAX));
                self.create_error_message(format!("Expected identifier, got {}", t.kind()));
                return None;
            }
            self.tokens.push(t.to_semantic_token(MACRO_TOKEN));

            let name = t.slice(self.src).to_string();

            let next = next_or_none!(self, "Expected content or , or ]")?;
            match next.kind() {
                AttributeToken::Content(inner_tokens) => {
                    let content = ParserContext::new(inner_tokens.iter().peekable(), self.diagnostics, self.tokens, self.src).parse();
                    attrs.push(Attribute { name, content });
                }
                AttributeToken::Comma => {
                    self.tokens.push(next.to_semantic_token(u32::MAX));
                }
                kind => {
                    self.tokens.push(next.to_semantic_token(u32::MAX));
                    self.create_error_message(format!("Unexpected token after identifier: {kind}"));
                    return None;
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
        let (semantic_token, result_text) = match t.kind() {
            ContentToken::String => {
                let text = t.slice(self.src).trim_matches('"');
                (t.to_semantic_token(STRING_TOKEN), text)
            }
            ContentToken::Value => (t.to_semantic_token(STRING_TOKEN), result_text),
            _ => {
                self.create_error_message("Expected value");
                return None;
            }
        };
        self.tokens.push(semantic_token);

        let t = next_or_none!(self, "Expected ')'")?;
        if t.kind() != &ContentToken::RightParenthesis {
            self.create_error_message(format!("Expected ')', got {}", t.kind()));
            return None;
        }
        self.tokens.push(t.to_semantic_token(u32::MAX));

        Some(result_text.to_string())
    }
}
