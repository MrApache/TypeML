mod ast;
mod cst;

pub use ast::*;
pub use cst::*;

use lexer_core::CstNode;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlParser;

impl RmlParser {
    #[must_use]
    pub fn build_cst(content: &str) -> CstNode<RmlNode> {
        let mut prev_line = 1;
        let mut prev_col = 1;
        let mut result = RmlParser::parse(Rule::file, content);
        if let Ok(mut tree) = result {
            CstNode::<RmlNode>::build_cst(&tree.next().unwrap(), content, &mut prev_line, &mut prev_col)
        } else {
            panic!("Error: {result:#?}");
        }
    }

    #[must_use]
    pub fn build_ast(content: &str) -> LayoutAst {
        let cst = Self::build_cst(content);
        build_layout_ast(&cst)
    }
}

#[cfg(test)]
mod tests {
    use crate::pest::RmlParser;
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/layout.rml");
    #[test]
    fn test() {
        let content = std::fs::read_to_string(PATH).unwrap();
        let ast = RmlParser::build_ast(&content);
        dbg!(ast);
    }
}
