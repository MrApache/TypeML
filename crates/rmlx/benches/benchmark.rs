use divan::Bencher;
use rmlx::{RmlxParser, Workspace};
use url::Url;

fn main() {
    divan::main();
}

#[divan::bench]
fn parse_ast() {
    const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
    let _ast = RmlxParser::build_ast(CONTENT);
}

#[divan::bench]
fn parse_cst() {
    const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
    let _cst = RmlxParser::build_cst(CONTENT);
}

#[divan::bench]
fn semantic_analyzis(bench: Bencher) {
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx");
    bench.with_inputs(|| {
        let w = Workspace::default();
        let url: Url = Url::parse(PATH).unwrap();
        (w, url)
    }).bench_values(|(mut w, url)| {
        w.load_single_model(&url);
    });
}
