#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
//#![allow(unused)]
//#![deny(clippy::unwrap_used)]

mod ast;
mod cst;
mod semantic;

pub use ast::{Count, CountEquality, SchemaAst, build_schema_ast};
pub use cst::RmlxNode;
use lexer_core::CstNode;
pub use pest::*;
use pest_derive::Parser;
pub use semantic::*;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlxParser;

impl RmlxParser {
    pub fn build_ast(content: &str) -> Result<SchemaAst, Error> {
        let cst = CstNode::new::<RmlxParser>(content, Rule::file).map_err(Error::PestError)?;
        Ok(build_schema_ast(&cst))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Root group not found")]
    RootGroupNotFound,

    #[error("Namespace {0} does not exist")]
    NamespaceNotFound(String),

    #[error("{0}")]
    IncorrectPattern(#[from] regex::Error),

    #[error("{0}")]
    LoadError(#[from] LoadError),

    #[error("{0}")]
    UrlError(String),

    #[error("{0}")]
    PestError(String),

    #[error("Element {0} not found")]
    ElementNotFound(String),

    #[error("Expression {0} not found")]
    ExpressionNotFound(String),

    #[error("Field {0} not found")]
    FieldNotFound(String),

    #[error("Type is not parsable")]
    TypeIsNotParsable,

    #[error("{0}")]
    ParseBool(#[from] std::str::ParseBoolError),

    #[error("{0}")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Invalid argument type. Current is {0}, but expected {1}")]
    InvalidArgumentType(String, String),

    #[error("Expression {0} is not allowed in {1} group")]
    ExpressionIsNotAllowedInGroup(String, String), //Expression, Group

    #[error("{0}::{1} is already defined")]
    AlreadyDefinedType(String, String),

    #[error("Duplicate field: {0}")]
    DuplicateField(String),

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Not enough elements from {group} group: {actual} found, need {expected}")]
    InsufficientElements {
        group: String,
        actual: u32,
        expected: Count,
    },

    #[error("Too many element from {group} group: found {actual}, need {expected}")]
    ExcessiveElements {
        group: String,
        actual: u32,
        expected: Count,
    },

    #[error("The element {0} is not unique")]
    NotUniqueElement(String),
}
