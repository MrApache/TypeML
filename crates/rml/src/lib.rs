#![allow(unused)]

use crate::ast::{LayoutAst, build_layout_ast};
use crate::cst::RmlNode;
use ::pest::Parser;
use lexer_core::CstNode;
use pest_derive::Parser;

mod ast;
mod cst;
mod model;

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
    use crate::RmlParser;
    use crate::model::LayoutModel;
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/layout.rml");
    #[test]
    fn test() {
        let content = std::fs::read_to_string(PATH).unwrap();
        let ast = RmlParser::build_ast(&content);
        LayoutModel::validate(ast, PATH);
    }
}
