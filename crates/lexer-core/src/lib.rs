#![allow(clippy::cast_possible_truncation)]

mod cst;
mod url;

pub use cst::*;
pub use url::*;

pub const KEYWORD_TOKEN: u32 = 0;
pub const PARAMETER_TOKEN: u32 = 1;
pub const STRING_TOKEN: u32 = 2;
pub const TYPE_TOKEN: u32 = 3;
pub const OPERATOR_TOKEN: u32 = 4;
pub const NUMBER_TOKEN: u32 = 5;
pub const COMMENT_TOKEN: u32 = 6;
pub const MACRO_TOKEN: u32 = 7;
pub const FUNCTION: u32 = 8;
