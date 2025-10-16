#[cfg(test)]
mod tests {
    use type_ml::{LayoutModel, RmlParser};
    const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/layout.tml");
    #[test]
    fn test() {
        let content = std::fs::read_to_string(PATH).unwrap();
        let ast = RmlParser::build_ast(&content);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }
}
