mod errors;
pub mod context;
pub use logos;

use logos::{Lexer, Logos, Span};

use crate::context::*;

pub trait TokenType {
    fn get_token_type(&self) -> u32;
}

#[derive(Default, Clone)]
pub struct Position {
    previous_token_line: u32,
    current_line: u32,

    previous_token_end_column: u32,
    current_column: u32,
}

impl Position {
    const fn get_delta_line(&mut self) -> u32 {
        let delta = self.current_line - self.previous_token_line;
        self.previous_token_line = self.current_line;
        delta
    }

    const fn get_delta_start(&mut self) -> u32 {
        self.current_column - self.previous_token_end_column
    }

    const fn set_end_column(&mut self, value: u32) {
        self.previous_token_end_column = value;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<T> {
    kind: T,
    span: Span,
    delta_line: u32, 
    delta_start: u32,
    length: u32,
}

impl<T> Token<T> {
    pub const fn kind(&self) -> &T {
        &self.kind
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub const fn delta_line(&self) -> u32 {
        self.delta_line
    }

    pub const fn delta_start(&self) -> u32 {
        self.delta_start
    }

    pub const fn length(&self) -> u32 {
        self.length
    }
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum DefaultContext {
    #[token("#", directive_context_callback)]
    Directive(Vec<Token<DirectiveContext>>),

    #[token(";", priority = 1)]
    CommentLine,

    #[token(";;", priority = 2)]
    CommentBlock,

    #[token("<", tag_context_callback, priority = 1)]
    TagStart(Vec<Token<TagContext>>),

    #[token("</", tag_context_callback, priority = 2)]
    TagCloseStart(Vec<Token<TagContext>>),

    #[regex(r"[^#;<]+")]
    Text,
}

pub struct RmlTokenStream<'a> {
    inner: Lexer<'a, DefaultContext>,
}

impl<'a> RmlTokenStream<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            inner: DefaultContext::lexer(content),
        }
    }

    pub fn next_token(&mut self) -> Result<Token<DefaultContext>, ()> {
        if let Some(token_kind) = self.inner.next() {
            let token_kind = token_kind?;

            if let DefaultContext::Text = token_kind {
                let delta_line = self.inner.extras.get_delta_line();
                let lines = self.inner.slice().matches('\n').count();
                self.inner.extras.current_line += lines as u32;

                Ok(Token {
                    kind: token_kind,
                    span: self.inner.span(),
                    delta_line,
                    delta_start: 0,
                    length: self.inner.span().len() as u32,
                })
            }
            else if let DefaultContext::Directive(tokens) = token_kind {
                Ok(Token {
                    kind: DefaultContext::Directive(tokens),
                    span: self.inner.span(),
                    delta_line: 0,
                    delta_start: 0,
                    length: self.inner.span().len() as u32,
                })
            }
            else {
                Ok(Token {
                    kind: DefaultContext::CommentLine,
                    span: self.inner.span(),
                    delta_line: 0,
                    delta_start: 0,
                    length: self.inner.span().len() as u32,
                })
            }

        }
        else {
            Err(())
        }
    }

    pub fn to_vec(mut self) -> Vec<Token<DefaultContext>> {
        let mut vec = vec![];

        while let Ok(token) = self.next_token() {
            vec.push(token);
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use logos::Logos;
    use crate::DefaultContext;

    #[test]
    fn test() {
        const CONTENT: &str = r#"
#import "../base.ron"
#import "../schema.ron"
#expressions "../file.rmlx" as expr

<Layout>
    <Node width="60px" height="100px"/>
    <Container id="Background">
        <Node/>
        <Image path="{expr:LoadIcon}">
        <Container id="Text">
            <Text self="{expr:Text}">
            <Tag attribute="{expr:Test}">
        </Container>
    </Container>
</Layout>
"#;
        let mut lexer = DefaultContext::lexer(CONTENT);

        while let Some(token) = lexer.next() {
            let slice = lexer.slice().trim();
            if let Ok(token) = &token && let DefaultContext::Text = token && slice.is_empty() {
                continue;
            }
            println!("{token:?} => {slice:?}");
        }
    }
}
