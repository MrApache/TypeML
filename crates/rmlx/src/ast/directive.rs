use crate::{ast::ParserContext, next_or_none, DirectiveToken};
use lexer_core::STRING_TOKEN;

#[derive(Debug)]
pub struct Directive {
    pub name: String,
    pub value: String,
}

impl ParserContext<'_, DirectiveToken> {
    pub fn parse(&mut self) -> Option<Directive> {
        self.consume_keyword();
        let name = self.consume_parameter()?; // consume directive name

        let t = next_or_none!(self, "Expected <...> value")?;
        match t.kind() {
            DirectiveToken::Value => {
                self.tokens.push(t.to_semantic_token(STRING_TOKEN));
                let value = self
                    .src
                    .get(t.span())
                    .map(trim_angle_brackets)
                    .unwrap()
                    .to_string();

                Some(Directive { name, value })
            }
            DirectiveToken::SyntaxError => {
                self.report_error(t, "Syntax error");
                None
            }
            kind => {
                self.report_error(t, &format!("Expected value, found {kind}"));
                None
            }
        }
    }
}

fn trim_angle_brackets(path: &str) -> &str {
    path.strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(path)
}
