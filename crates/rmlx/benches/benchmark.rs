use rmlx::SchemaAst;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench]
fn parse_ast() {
    let path = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx");
    let content = std::fs::read_to_string(path).expect("Failed to read file");
    let _ast = SchemaAst::new(&content);
}
