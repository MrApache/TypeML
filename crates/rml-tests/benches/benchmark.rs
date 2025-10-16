use divan::Bencher;
use lexer_core::CstNode;
use rml::{LayoutModel, RmlParser};
use rmlx::{AnalysisWorkspace, RmlxNode, RmlxParser, Rule, build_schema_ast};
use url::Url;

fn main() {
    divan::main();
}

#[divan::bench]
fn parse_ast(bench: Bencher) {
    bench
        .with_inputs(|| {
            const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
            CstNode::new::<RmlxParser>(CONTENT, Rule::file).unwrap()
        })
        .bench_values(|cst| build_schema_ast(&cst));
}

#[divan::bench]
fn parse_cst() {
    const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
    let _: CstNode<RmlxNode> = CstNode::new::<RmlxParser>(CONTENT, Rule::file).unwrap();
}

#[divan::bench]
fn semantic_analysis(bench: Bencher) {
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx");
    bench
        .with_inputs(|| {
            let url = Url::from_file_path(PATH).unwrap();
            AnalysisWorkspace::new(url)
        })
        .bench_values(|w| {
            let _ = w.run();
        });
}

#[divan::bench]
fn rml_full_analysis(bench: Bencher) {
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/layout.rml");
    bench
        .with_inputs(|| {
            let content = std::fs::read_to_string(PATH).unwrap();
            RmlParser::build_ast(&content).unwrap()
        })
        .bench_values(|ast| {
            LayoutModel::validate(ast, PATH).unwrap();
        });
}
