use rmlx::{Attribute, Count, Directive, Expression, Group, SchemaAst, Type, TypeDefinition};

#[rustfmt::skip]
fn assert_directive(directives: &[Directive], name: &str, value: &str) {
    let found = directives.iter().any(|d| d.name == name && d.value == value);
    assert!(found,"Directive not found: expected name = {name}, value = {value}. Found: {directives:#?}");
}

#[rustfmt::skip]
fn assert_attribute(attrs: &[Attribute], name: &str, content: Option<&str>) {
    let found = attrs.iter().any(|a| a.name() == name && a.content().as_deref() == content);
    assert!(found, "Attribute not found: expected name = {name}, content = {content:#?}. Found: {attrs:#?}");
}

#[rustfmt::skip]
fn assert_group(
    groups: &[Group],
    name: &str,
    count: Option<&Count>,
    refs: &[(&str, Option<Count>, bool)],
    attributes: &[(&str, Option<&str>)],
) {
    assert!(groups.iter().any(|group| {
        if group.name() != name {
            return false;
        }

        assert_eq!(group.name(), name, "Group name mismatch: expected {name}, found {}", group.name());
        assert_eq!(group.count().as_ref(), count, "Group count mismatch for group {name}");

        for (name, content) in attributes {
            assert_attribute(group.attributes(), name, *content);
        }

        assert_eq!(group.groups().len(), refs.len(), "Number of child groups mismatch for group {name}");

        refs.iter().zip(group.groups()).for_each(|((name, count, unique), ref_group)| {
            assert_eq!(ref_group.name(), *name, "RefGroup name mismatch: expected {name}, found {}", ref_group.name());
            assert_eq!(ref_group.count().as_ref(), count.as_ref(), "RefGroup count mismatch for {name}");
            assert_eq!(ref_group.unique(), *unique, "RefGroup uniqueness mismatch for {name}");
        });

        true
    }), "Group '{name}' not found");
}

fn assert_typedef(
    typedefs: &[TypeDefinition],
    keyword: &str,
    name: &str,
    fields: &[(&str, Type)],
    bind: Option<(Option<&str>, &str)>,
    attributes: &[(&str, Option<&str>)],
) {
    assert!(typedefs.iter().any(|typedef| {
        if typedef.keyword() != keyword || typedef.name() != name {
            return false;
        }

        fields.iter().zip(typedef.fields()).for_each(|((name, ty), field)| {
            assert_eq!(field.name(), *name, "Field name mismatch: expected {name}, found {}", field.name());
            assert_eq!(field.ty(), ty, "Field type mismatch: expected {ty:#?}, found {:#?}", field.ty());
        });

        assert!(match (bind, typedef.bind()) {
            (None, None) => true,
            (Some((ns_a, name_a)), Some(ref_type)) => {
                ns_a == ref_type.namespace() && name_a == ref_type.name()
            }
            _ => false,
        }, "element bind type mismatch: expected {bind:#?}, found {:#?}", typedef.bind());

        for (name, content) in attributes {
            assert_attribute(typedef.attributes(), name, *content);
        }

        true
    }));
}

#[test]
fn ast() {
    let path = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/base.rmlx");
    let content = std::fs::read_to_string(path).expect("Failed to read file");
    let ast = SchemaAst::new(&content);

    assert_directive(ast.directives(), "namespace", "base");
    assert_group(
        ast.groups(),
        "Container",
        None,
        &[
            ("Component", Some(Count::Range(0..u32::MAX)), true),
            ("Container", Some(Count::Range(0..1)), false),
            ("Template", Some(Count::Range(0..1)), false),
        ],
        &[("Description", Some("Entity"))],
    );

    assert_group(
        ast.groups(),
        "Root",
        Some(&Count::Single(1)),
        &[
            ("Component", Some(Count::Range(0..u32::MAX)), true),
            ("Container", Some(Count::Range(0..1)), false),
        ],
        &[("Description", Some("Main group"))],
    );

    assert_group(
        ast.groups(),
        "Template",
        None,
        &[("Container", Some(Count::Single(1)), false)],
        &[("Description", Some("Template"))],
    );

    assert_typedef(
        ast.types(),
        "element",
        "Layout",
        &[],
        Some((None, "Root")),
        &[],
    );

    assert_typedef(
        ast.types(),
        "element",
        "ItemTemplate",
        &[],
        Some((None, "Template")),
        &[],
    );

    assert_typedef(
        ast.types(),
        "element",
        "Test",
        &[
            ("target", Type::Simple("String".into())),
            ("path", Type::Simple("String".into()))
        ],
        Some((None, "Template")),
        &[],
    );
}
