use crate::{semantic::{
    element::UnresolvedElementSymbol, enumeration::UnresolvedEnumSymbol, group::UnresolvedGroupSymbol,
    structure::UnresolvedStructSymbol, symbol::SymbolKind,
}, utils::to_url, AnalysisWorkspace, CustomType, RmlxParser, SchemaAst, TypeResolver};

pub struct UnresolvedSchema {
    namespace: Option<String>,
    structs: Vec<UnresolvedStructSymbol>,
    enums: Vec<UnresolvedEnumSymbol>,
    groups: Vec<UnresolvedGroupSymbol>,
    elements: Vec<UnresolvedElementSymbol>,
}

impl UnresolvedSchema {
    pub fn new(source: &str, path: &str, workspace: &mut AnalysisWorkspace) -> Self {
        let ast = RmlxParser::build_ast(source);
        let directive_result = process_directives(&ast);
        directive_result.uses.iter().for_each(|u| {
            workspace.load_single_model(&to_url(path, u).unwrap());
        });

        let enums = ast
            .custom_types
            .iter()
            .filter(|t| t.is_enum())
            .map(CustomType::unwrap_enum)
            .map(UnresolvedEnumSymbol::new)
            .collect::<Vec<_>>();

        let structs = ast
            .custom_types
            .iter()
            .filter(|t| t.is_struct())
            .map(CustomType::unwrap_struct)
            .map(UnresolvedStructSymbol::new)
            .collect::<Vec<_>>();

        let groups = ast
            .custom_types
            .iter()
            .filter(|t| t.is_group())
            .map(CustomType::unwrap_group)
            .map(UnresolvedGroupSymbol::new)
            .collect::<Vec<_>>();

        let elements = ast
            .custom_types
            .iter()
            .filter(|t| t.is_element())
            .map(CustomType::unwrap_element)
            .map(UnresolvedElementSymbol::new)
            .collect::<Vec<_>>();

        UnresolvedSchema {
            namespace: directive_result.namespace,
            structs,
            enums,
            groups,
            elements,
        }
    }

    pub fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> Vec<SymbolKind> {
        let mut symbols = vec![];
        self.structs.retain_mut(|s| {
            let result = s.resolve(workspace);
            if result {
                symbols.push(SymbolKind::Struct(s.as_resolved_type()));
            }
            !result
        });

        self.enums.retain_mut(|e| {
            let result = e.resolve(workspace);
            if result {
                symbols.push(SymbolKind::Enum(e.as_resolved_type()));
            }
            !result
        });

        self.groups.retain_mut(|g| {
            let result = g.resolve(workspace);
            if result {
                symbols.push(SymbolKind::Group(g.as_resolved_type()));
            }
            !result
        });

        self.elements.retain_mut(|e| {
            let result = e.resolve(workspace);
            if result {
                symbols.push(SymbolKind::Element(e.as_resolved_type()));
            }
            !result
        });

        symbols
    }

    pub fn is_empty(&self) -> bool {
        self.enums.is_empty() && self.structs.is_empty() && self.groups.is_empty() && self.elements.is_empty()
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
}

struct DirectiveResult {
    namespace: Option<String>,
    uses: Vec<String>,
    errors: Vec<String>,
}

fn process_directives(ast: &SchemaAst) -> DirectiveResult {
    let mut namespace: Option<String> = None;
    let mut uses = Vec::new();
    let mut errors = Vec::new();

    ast.directives.iter().for_each(|d| match d.name.as_str() {
        "namespace" => {
            if let Some(ns) = &namespace {
                errors.push(format!("Duplicate namespace directive found: {ns}"));
            } else {
                namespace.clone_from(&d.value);
            }
        }
        "use" => {
            let value = d.value.clone().unwrap();
            uses.push(value);
        }
        other => errors.push(format!("Unknown directive: {other}")),
    });

    DirectiveResult {
        namespace,
        uses,
        errors,
    }
}
