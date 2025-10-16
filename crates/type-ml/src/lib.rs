#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
//#![allow(unused)]

mod analyzer;
mod cst;
mod model;
mod resolved;
mod unresolved;

use crate::analyzer::RmlAnalyzer;
pub use crate::model::LayoutModel;
pub use crate::unresolved::LayoutAst;
use lexer_core::CstNode;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlParser;

impl RmlParser {
    pub fn build_ast(content: &str) -> Result<LayoutAst, type_ml_definitions::Error> {
        let cst = CstNode::new::<RmlParser>(content, Rule::file).map_err(type_ml_definitions::Error::PestError)?;
        Ok(LayoutAst::build(&cst))
    }
}

pub trait NodeResolver<T> {
    fn resolve(&self, analyzer: &RmlAnalyzer) -> T;
}
