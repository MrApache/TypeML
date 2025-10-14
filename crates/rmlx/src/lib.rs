#![allow(clippy::missing_panics_doc)]
#![allow(unused)]

mod ast;
mod cst;
mod semantic;

pub use ast::{SchemaAst, build_schema_ast};
pub use cst::RmlxNode;
use lexer_core::CstNode;
pub use pest::*;
use pest_derive::Parser;
pub use semantic::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlxParser;

impl RmlxParser {
    #[must_use]
    pub fn build_cst(content: &str) -> CstNode<RmlxNode> {
        let mut prev_line = 1;
        let mut prev_col = 1;
        let mut result = RmlxParser::parse(Rule::file, content);
        if let Ok(mut tree) = result {
            CstNode::<RmlxNode>::build_cst(&tree.next().unwrap(), content, &mut prev_line, &mut prev_col)
        } else {
            panic!("Error: {result:#?}");
        }
    }

    #[must_use]
    pub fn build_ast(content: &str) -> SchemaAst {
        let cst = Self::build_cst(content);
        build_schema_ast(&cst)
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnalysisWorkspace, RmlAnalyzer};
    use url::Url;

    #[test]
    fn test() {
        let path = "D:\\Projects\\rml\\examples\\schema.rmlx";
        let url = Url::from_file_path(path).unwrap();
        let mut workspace = AnalysisWorkspace::new(url).run();
        let mut rml = RmlAnalyzer::new(workspace.model.clone());
        let _allowed = rml.is_allowed_element(None, "Node");
        rml.next_state(None, "Node");
        let _allowed_attribute = rml.is_valid_attribute("left", "10px");
        let _allowed_generic = rml.is_valid_attribute("aspect_ratio", "Some(10)");
        println!();
    }
}
