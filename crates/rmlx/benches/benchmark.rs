use rmlx::RmlxParser;

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
