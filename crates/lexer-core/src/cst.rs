#![allow(unused)]

use pest::iterators::Pair;
use std::fmt::Debug;
use std::hash::Hash;

pub trait CstKind: Debug + Clone + Copy + PartialEq + Eq {
    type Rule: Copy + Debug + Eq + Hash + Ord;
    fn map_rule_to_cst_kind(rule: Self::Rule) -> Self;
}

#[derive(Debug, Clone)]
pub struct CstNode<K: CstKind> {
    pub kind: K,
    pub text: String,
    pub children: Vec<CstNode<K>>,
    pub start: usize,     // абсолютная позиция в файле
    pub end: usize,       // абсолютная позиция в файле
    pub delta_line: u32,  // строки от предыдущего токена
    pub delta_start: u32, // смещение в строке
}

impl<K: CstKind> CstNode<K> {
    pub fn build_cst(pair: &Pair<K::Rule>, source: &str, prev_line: &mut u32, prev_col: &mut u32) -> Self {
        let span = pair.as_span();
        let (start_line_1, start_col_1) = span.start_pos().line_col();
        let (end_line_1, end_col_1) = span.end_pos().line_col();

        // LSP uses 0-based indexing
        let start_line = (start_line_1 - 1) as u32;
        let end_line = (end_line_1 - 1) as u32;

        // Compute UTF-16 columns
        let byte_start = span.start();
        let byte_end = span.end();

        let line_start_byte = source[..byte_start].rfind('\n').map_or(0, |pos| pos + 1);

        let start_col_utf16 = source[line_start_byte..byte_start].encode_utf16().count() as u32;
        let end_col_utf16 = source[line_start_byte..byte_end].encode_utf16().count() as u32;

        // --- delta calculation (relative to previous token start) ---
        let delta_line = start_line.saturating_sub(*prev_line);

        let delta_start = if delta_line == 0 {
            start_col_utf16.saturating_sub(*prev_col)
        } else {
            start_col_utf16
        };

        // Теперь prev обновляем ПОСЛЕ того, как вычислили дельты
        *prev_line = start_line;
        *prev_col = start_col_utf16;

        let kind = K::map_rule_to_cst_kind(pair.as_rule());

        // рекурсивно строим детей
        let children: Vec<CstNode<K>> = pair
            .clone()
            .into_inner()
            .map(|inner| Self::build_cst(&inner, source, prev_line, prev_col))
            .collect();

        CstNode {
            kind,
            text: span.as_str().trim().to_string(),
            children,
            start: byte_start,
            end: byte_end,
            delta_line,
            delta_start,
        }
    }
}
