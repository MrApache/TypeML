use divan::Bencher;
use rmlx::{build_schema_ast, RmlxParser, AnalysisWorkspace};
use url::Url;

fn main() {
    divan::main();
}

#[divan::bench]
fn parse_ast(bench: Bencher) {
    bench.with_inputs(|| {
        const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
        RmlxParser::build_cst(CONTENT)
    }).bench_values(|cst| build_schema_ast(&cst))
}

#[divan::bench]
fn parse_cst() {
    const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
    let _cst = RmlxParser::build_cst(CONTENT);
}

#[divan::bench]
fn semantic_analysis(bench: Bencher) {
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx");
    bench.with_inputs(|| {
        let w = AnalysisWorkspace::default();
        let url: Url = Url::from_file_path(PATH).unwrap();
        (w, url)
    }).bench_values(|(mut w, url)| {
        w.load_single_model(&url);
    });
}
