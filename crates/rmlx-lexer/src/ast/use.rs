use lexer_utils::STRING_TOKEN;
use crate::{next_or_none, ast::ParserContext, UseToken};

pub struct Use {
    pub path: String,
}

impl<'s> ParserContext<'s, UseToken> {
    pub fn parse(&mut self) -> Option<Use> {
        self.consume_keyword();

        let t = next_or_none!(self, "Expected <...> path")?;
        match t.kind() {
            UseToken::Path => self.tokens.push(t.to_semantic_token(STRING_TOKEN)),
            UseToken::SyntaxError => {
                self.report_error(t, "Syntax error");
                return None;
            }
            _ => {
                self.report_error(t, &format!("Expected path, found {}", t.kind()));
                return None;
            },
        }

        let path = self.src
            .get(t.span())
            .map(trim_angle_brackets)
            .unwrap()
            .to_string();


        Some(Use { path })
    }
}

fn trim_angle_brackets(path: &str) -> &str {
    path.strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(path)
}
