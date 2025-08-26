mod error;

use lexer_utils::Token;
use rml_lexer::{
    context::DirectiveContext,
    logos::{Source, Span},
    MarkupTokens, RmlTokenStream
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Directive {
    Import { path: String },
    Expressions { path: String, alias: String },
}

pub struct RmlParser<'a> {
    _file: &'a str,
    content: &'a str,
    lexer: RmlTokenStream<'a>,
}

impl<'a> RmlParser<'a> {
    pub fn new(content: &'a str, file: &'a str) -> Self {
        Self {
            _file: file,
            content,
            lexer: RmlTokenStream::new(content),
        }
    }

    fn get_slice(&self, span: Span) -> &str {
        self.content.slice(span).unwrap()
    }

    pub fn read_directives(&mut self) -> Result<Vec<Directive>, ()> {
        let mut directives = vec![];

        while let Ok(token_kind) = self.lexer.next_token() {
            if let MarkupTokens::Directive(tokens) = token_kind {
                directives.push(self.parse_directive(&tokens)?);
            }
        }
        Ok(directives)
    }

    fn parse_directive(&self, tokens: &[Token<DirectiveContext>]) -> Result<Directive, ()> {
        let mut iter = tokens.iter();
        iter.next(); //TODO skip token 'Start'
        match iter.next() {
            Some(token) => {
                match token.kind() {
                    DirectiveContext::Expression => {
                        let path = self.try_read_directive_path(iter.next())?;
                        let alias = self.try_read_expr_alias(iter.next(), iter.next())?;
                        Ok(Directive::Expressions { path, alias })
                    },
                    DirectiveContext::Import => {
                        let path = self.try_read_directive_path(iter.next())?;
                        Ok(Directive::Import { path })
                    },
                    _ => unreachable!(), //An error if a token is missing
                }
            }
            _ => unreachable!(), //An error if a token is missing
        }
    }

    fn try_read_directive_path(
        &self,
        token: Option<&Token<DirectiveContext>>,
    ) -> Result<String, ()> {
        if token.is_none() {
            return Err(()); // Token 'Path' is missing
        }
        let token = token.unwrap();
        if token.kind() != &DirectiveContext::Path {
            return Err(()); // Unexpected token 'Path'
        }
        Ok(self.get_slice(token.span()).trim_matches('"').to_string())
    }

    fn try_read_expr_alias(
        &self,
        token_as: Option<&Token<DirectiveContext>>,
        token_alias: Option<&Token<DirectiveContext>>,
    ) -> Result<String, ()> {
        if token_as.is_none() {
            return Err(()); // Token 'As' is missing
        }

        if token_alias.is_none() {
            return Err(()); // Token 'Alias' is missing
        }

        let token_alias = token_alias.unwrap();
        Ok(self.get_slice(token_alias.span()).to_string())
    }

    pub fn parse(&mut self) -> Result<(), ()> {
        let imports = self.read_directives()?;
        self.lexer = RmlTokenStream::new(self.content);

        println!("{imports:#?}");

        Ok(())

        //let content = std::fs::read_to_string("schema.ron").unwrap();
        //let entries: Vec<SchemaEntry> = ron::de::from_str(&content).unwrap();
        //let schema: Schema = entries.into();

        //println!("{schema:#?}");
    }
}

#[cfg(test)]
mod tests {
    use rml_lexer::RmlTokenStream;
    use crate::parser::RmlParser;

    #[test]
    fn test() {
        const CONTENT: &str =
r#"<Node attribute="{{ x:10, y:20 }}"/>"#;

        let _tokens = RmlTokenStream::new(CONTENT).to_vec();
        let mut parser = RmlParser::new(CONTENT, "");
        let directives = parser.read_directives();
        println!("{directives:#?}");
    }
}
