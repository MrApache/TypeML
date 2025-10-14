use divan::Bencher;
use rmlx::{AnalysisWorkspace, RmlAnalyzer, RmlxParser, build_schema_ast};
use url::Url;

fn main() {
    divan::main();
}

#[divan::bench]
fn parse_ast(bench: Bencher) {
    bench
        .with_inputs(|| {
            const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
            RmlxParser::build_cst(CONTENT)
        })
        .bench_values(|cst| build_schema_ast(&cst));
}

#[divan::bench]
fn parse_cst() {
    const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
    let _cst = RmlxParser::build_cst(CONTENT);
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

/*
#[divan::bench]
fn rml_analysis(bench: Bencher) {
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx");
    bench
        .with_inputs(|| {
            let url = Url::from_file_path(PATH).unwrap();
            let w = AnalysisWorkspace::new(url);
            RmlAnalyzer::new(w.model())
        })
        .bench_values(|rml| rml.is_allowed_element());
}
*/
