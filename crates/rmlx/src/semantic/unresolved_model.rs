use crate::{
    semantic::{
        element::UnresolvedElementSymbol, enumeration::UnresolvedEnumSymbol, group::UnresolvedGroupSymbol, structure::UnresolvedStructSymbol, symbol::SymbolKind
    },
    utils::to_url,
    SchemaAst, TypeResolver, Workspace,
};

pub struct UnresolvedSchemaModel {
    namespace: Option<String>,
    structs: Vec<UnresolvedStructSymbol>,
    enums: Vec<UnresolvedEnumSymbol>,
    groups: Vec<UnresolvedGroupSymbol>,
    elements: Vec<UnresolvedElementSymbol>,
}

impl UnresolvedSchemaModel {
    pub fn new(ast: SchemaAst, path: &str, workspace: &mut Workspace) -> Self {
        let directive_result = process_directives(&ast); //TODO errors
        directive_result.uses.iter().for_each(|u| {
            let url = to_url(path, u).unwrap();
            workspace.load_single_model(url);
        });

        let enums = ast.enumerations().iter().map(UnresolvedEnumSymbol::new).collect::<Vec<_>>();
        let structs = ast.types().iter().filter(|f| f.keyword() == "struct").map(UnresolvedStructSymbol::new).collect::<Vec<_>>();
        let mut groups = ast.groups().iter().map(UnresolvedGroupSymbol::new).collect::<Vec<_>>();
        groups.extend(ast.extendable_groups().iter().map(UnresolvedGroupSymbol::new_extendable));
        let elements = ast.types().iter().filter(|f| f.keyword() == "element").map(UnresolvedElementSymbol::new).collect::<Vec<_>>();

        UnresolvedSchemaModel {
            namespace: directive_result.namespace,
            structs,
            enums,
            groups,
            elements,
        }
    }

    pub fn resolve(&mut self, workspace: &Workspace) -> Vec<SymbolKind> {
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
        self.enums.is_empty()
            && self.structs.is_empty()
            && self.groups.is_empty()
            && self.elements.is_empty()
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
}

struct DirectiveResult<'s> {
    namespace: Option<String>,
    uses: Vec<&'s str>,
    errors: Vec<String>,
}

fn process_directives(ast: &SchemaAst) -> DirectiveResult {
    let mut namespace: Option<String> = None;
    let mut uses = Vec::new();
    let mut errors = Vec::new();

    ast.directives().iter().for_each(|d| match d.name() {
        "namespace" => {
            if namespace.is_some() {
                errors.push(format!("Duplicate namespace directive found: {}", d.value()));
            } else {
                namespace = Some(d.value().to_string());
            }
        }
        "use" => uses.push(d.value()),
        other => errors.push(format!("Unknown directive: {other}")),
    });

    DirectiveResult {
        namespace,
        uses,
        errors,
    }
}
