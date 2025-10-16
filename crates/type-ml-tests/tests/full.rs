#[cfg(test)]
mod tests {
    use type_ml::{LayoutAst, LayoutModel, RmlParser};

    fn load(path: &str) -> Result<LayoutAst, type_ml_definitions::Error> {
        let content = std::fs::read_to_string(path).unwrap();
        RmlParser::build_ast(&content)
    }

    #[test]
    fn full() {
        const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/layout.tml");
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn unresolved_type() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/0_unresolved_type/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::UnresolvedType(_, _)
        ));
    }

    #[test]
    fn root_not_found() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/1_root_not_found/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::RootGroupNotFound
        ));
    }

    #[test]
    fn namespace_not_found() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/2_namespace_not_found/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::NamespaceNotFound(_)
        ));
    }

    #[test]
    fn pest_error() {
        const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/errors/3_pest_error/layout.tml");
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), type_ml_definitions::Error::PestError(_)));
    }

    #[test]
    fn element_not_found() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/4_element_not_found/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::ElementNotFound(_)
        ));
    }

    #[test]
    fn expression_not_found() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/5_expression_not_found/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::ExpressionNotFound(_)
        ));
    }

    #[test]
    fn field_not_found() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/6_field_not_found/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::FieldNotFound(_)
        ));
    }

    #[test]
    fn parse_bool() {
        const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/errors/7_parse_bool/layout.tml");
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), type_ml_definitions::Error::ParseBool(_)));
    }

    #[test]
    fn parse_float() {
        const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/errors/8_parse_float/layout.tml");
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), type_ml_definitions::Error::ParseFloat(_)));
    }

    #[test]
    fn parse_int() {
        const PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/errors/9_parse_int/layout.tml");
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), type_ml_definitions::Error::ParseInt(_)));
    }

    #[test]
    fn invalid_argument_type() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/10_invalid_argument_type/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::InvalidArgumentType(_, _)
        ));
    }

    #[test]
    fn expression_is_not_allowed() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/11_expression_is_not_allowed/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::ExpressionIsNotAllowedInGroup(_, _)
        ));
    }

    #[test]
    fn already_defined_type() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/12_already_defined_type/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::AlreadyDefinedType(_, _)
        ));
    }

    #[test]
    fn duplicate_field() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/13_duplicate_field/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::DuplicateField(_)
        ));
    }

    #[test]
    fn missing_required_field() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/14_missing_required_field/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::MissingRequiredField(_)
        ));
    }

    #[test]
    fn insufficient_elements() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/15_insufficient_elements/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::InsufficientElements {
                group: _,
                actual: _,
                expected: _
            }
        ));
    }

    #[test]
    fn excessive_elements() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/16_excessive_elements/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::ExcessiveElements {
                group: _,
                actual: _,
                expected: _
            }
        ));
    }

    #[test]
    fn not_unique_element() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/17_not_unique_element/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::NotUniqueElement(_)
        ));
    }

    #[test]
    fn cant_extend_group() {
        const PATH: &str = concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "examples/errors/18_cant_extend_group/layout.tml"
        );
        let ast = load(PATH);
        assert!(ast.is_ok(), "{}", ast.unwrap_err());
        let result = LayoutModel::validate(ast.unwrap(), PATH);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            type_ml_definitions::Error::CantExtendGroup(_)
        ));
    }

    //TODO Incorrect pattern
    //TODO Load error
    //TODO Url error
    //TODO Type is not parsable
}
