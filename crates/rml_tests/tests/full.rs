#[cfg(test)]
mod tests {
    use rml::{LayoutModel, RmlParser};
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/layout.rml");
    #[test]
    fn test() {
        let content = std::fs::read_to_string(PATH).unwrap();
        let ast = RmlParser::build_ast(&content);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        println!("{:#?}", result);
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }
}
