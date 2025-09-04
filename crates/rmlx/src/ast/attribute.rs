use crate::{next_or_none, AttributeToken, ContentToken, ParserContext};

#[derive(Debug)]
pub struct Attribute {
    name: String,
    content: Option<String>,
}

impl Attribute {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn content(&self) -> &Option<String> {
        &self.content
    }
}

impl ParserContext<'_, AttributeToken> {
    pub fn parse(&mut self) -> Option<Vec<Attribute>> {
        self.consume_keyword();
        self.consume_left_square_bracket()?;

        let mut attrs = Vec::new();
        loop {
            let name_token = next_or_none!(self)?;
            match name_token.kind() {
                AttributeToken::Comma => continue,
                AttributeToken::RightSquareBracket => break,
                AttributeToken::Identifier => {} // continue
                AttributeToken::SyntaxError => self.report_error("Syntax error"),
                kind => self.report_error(&format!("Expected identifier, got {kind}")),
            }
            let name = name_token.slice(self.src).to_string();

            let next = next_or_none!(self, "Expected content or , or ]")?;
            match next.kind() {
                AttributeToken::Content(inner_tokens) => {
                    let content = ParserContext::new(
                        inner_tokens.iter().peekable(),
                        self.diagnostics,
                        self.src,
                    )
                    .parse();
                    attrs.push(Attribute { name, content });
                }
                AttributeToken::Comma => {}, // continue
                AttributeToken::RightSquareBracket => break,
                AttributeToken::SyntaxError => self.report_error("Syntax error"),
                kind => self.report_error(&format!("Unexpected token after identifier: {kind}")),
            }
        }

        Some(attrs)
    }

    fn consume_left_square_bracket(&mut self) -> Option<()> {
        let left_square = next_or_none!(self)?;
        match left_square.kind() {
            AttributeToken::LeftSquareBracket => return Some(()),
            AttributeToken::SyntaxError => self.report_error("Syntax error"),
            kind => self.report_error(&format!("Expected '[', got {kind}")),
        }

        None
    }
}

impl ParserContext<'_, ContentToken> {
    pub fn parse(&mut self) -> Option<String> {
        self.consume_keyword();

        let t = next_or_none!(self, "Expected String or Value")?;
        let result_text = t.slice(self.src);
        let result_text = match t.kind() {
            ContentToken::String => t.slice(self.src).trim_matches('"'),
            ContentToken::Value => result_text,
            ContentToken::SyntaxError => {
                self.report_error("Syntax error");
                ""
            }
            _ => {
                self.report_error("Expected value");
                ""
            }
        };

        self.consume_right_par()?;

        Some(result_text.to_string())
    }

    fn consume_right_par(&mut self) -> Option<()> {
        let right_par = next_or_none!(self, "Expected ')'")?;
        match right_par.kind() {
            ContentToken::RightParenthesis => return Some(()),
            ContentToken::SyntaxError => self.report_error("Syntax error"),
            kind => self.report_error(&format!("Expected ')', got {kind}")),
        }

        None
    }
}
