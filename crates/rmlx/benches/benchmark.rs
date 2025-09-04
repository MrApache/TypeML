use rmlx::SchemaAst;

fn main() {
    divan::main();
}

#[divan::bench]
fn parse_ast() {
    const CONTENT: &str = include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx"));
    let _ast = SchemaAst::new(CONTENT);
}
