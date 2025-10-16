use crate::ast::CustomType;
use crate::semantic::expression::UnresolvedExpressionSymbol;
use crate::{
    AnalysisWorkspace, RmlxParser, SchemaAst, TypeResolver,
    semantic::{
        element::UnresolvedElementSymbol, enumeration::UnresolvedEnumSymbol, group::UnresolvedGroupSymbol,
        structure::UnresolvedStructSymbol, symbol::SymbolKind,
    },
};
use lexer_core::to_url;

#[derive(Debug)]
pub struct UnresolvedSchema {
    namespace: Option<String>,
    groups: Vec<UnresolvedGroupSymbol>,
    expressions: Vec<UnresolvedExpressionSymbol>,

    enums: Vec<UnresolvedEnumSymbol>,
    structs: Vec<UnresolvedStructSymbol>,
    elements: Vec<UnresolvedElementSymbol>,
}

impl UnresolvedSchema {
    pub fn new(source: &str, path: &str, workspace: &mut AnalysisWorkspace) -> Result<Self, crate::Error> {
        let ast = RmlxParser::build_ast(source)?;
        let directive_result = process_directives(&ast);
        directive_result.uses.iter().try_for_each(|u| {
            workspace.load_single_model(&to_url(path, u).map_err(crate::Error::UrlError)?)?;
            Ok::<_, crate::Error>(())
        })?;

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

        let expressions = ast
            .custom_types
            .iter()
            .filter(|t| t.is_expression())
            .map(CustomType::unwrap_expression)
            .map(UnresolvedExpressionSymbol::new)
            .collect::<Vec<_>>();

        Ok(UnresolvedSchema {
            namespace: directive_result.namespace,
            structs,
            enums,
            groups,
            elements,
            expressions,
        })
    }

    pub fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> Result<Vec<SymbolKind>, crate::Error> {
        let mut symbols = vec![];
        self.structs.try_retain_mut(|s| {
            let result = s.resolve(workspace)?;
            if result {
                symbols.push(SymbolKind::Struct(s.as_resolved_type()));
            }
            Ok::<bool, crate::Error>(!result)
        })?;

        self.enums.try_retain_mut(|e| {
            let result = e.resolve(workspace)?;
            if result {
                symbols.push(SymbolKind::Enum(e.as_resolved_type()));
            }
            Ok::<bool, crate::Error>(!result)
        })?;

        self.groups.try_retain_mut(|g| {
            let result = g.resolve(workspace)?;
            if result {
                symbols.push(SymbolKind::Group(g.as_resolved_type()));
            }
            Ok::<bool, crate::Error>(!result)
        })?;

        self.elements.try_retain_mut(|e| {
            let result = e.resolve(workspace)?;
            if result {
                symbols.push(SymbolKind::Element(e.as_resolved_type()));
            }
            Ok::<bool, crate::Error>(!result)
        })?;

        self.expressions.try_retain_mut(|e| {
            let result = e.resolve(workspace)?;
            if result {
                symbols.push(SymbolKind::Expression(e.as_resolved_type()));
            }
            Ok::<bool, crate::Error>(!result)
        })?;

        Ok(symbols)
    }

    pub fn is_empty(&self) -> bool {
        self.enums.is_empty()
            && self.structs.is_empty()
            && self.groups.is_empty()
            && self.elements.is_empty()
            && self.expressions.is_empty()
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn next_unresolved(&self) -> Option<&str> {
        if !self.groups.is_empty() {
            return self.groups.first().map(|g| g.identifier());
        }

        if !self.expressions.is_empty() {
            return self.expressions.first().map(|g| g.identifier());
        }

        if !self.enums.is_empty() {
            return self.enums.first().map(|g| g.identifier());
        }

        if !self.structs.is_empty() {
            return self.structs.first().map(|g| g.identifier());
        }

        if !self.elements.is_empty() {
            return self.elements.first().map(|g| g.identifier());
        }

        None
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
            let value = d.value.clone().expect("Unreachable!");
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

trait TryRetainMut<T> {
    fn try_retain_mut<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: FnMut(&mut T) -> Result<bool, E>;
}

impl<T> TryRetainMut<T> for Vec<T> {
    fn try_retain_mut<F, E>(&mut self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut T) -> Result<bool, E>,
    {
        let mut original_len = self.len();
        let mut i = 0;

        while i < original_len {
            if f(&mut self[i])? {
                i += 1;
            } else {
                // Swap with last element and reduce length
                self.swap_remove(i);
                original_len -= 1;
                // Don't increment i since we need to check the swapped element
            }
        }

        Ok(())
    }
}
