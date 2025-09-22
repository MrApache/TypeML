use crate::{
    semantic::symbol::{Symbol, SymbolRef},
    TypeResolver, UnresolvedType, Workspace,
};
use std::collections::HashMap;

pub struct StructSymbol {
    identifier: String,
    fields: Vec<ResolvedField>,
    metadata: HashMap<String, Option<String>>,
}

#[derive(Clone)]
pub struct ResolvedField {
    identifier: String,
    ty: SymbolRef,
}

pub struct UnresolvedStructField {
    identifier: String,
    ty: UnresolvedType,
}

impl UnresolvedStructField {
    pub fn new(f: &crate::ast::Field) -> UnresolvedStructField {
        let identifier = f.name().to_string();
        let simple_type = f.ty().take_simple();
        UnresolvedStructField {
            identifier,
            ty: UnresolvedType {
                namespace: None,
                identifier: simple_type,
            },
        }
    }
}

pub struct UnresolvedStructSymbol {
    pub identifier: String,
    pub fields: Vec<UnresolvedStructField>,
    pub metadata: HashMap<String, Option<String>>,
    pub resolved: Vec<ResolvedField>,
}

impl UnresolvedStructSymbol {
    pub fn new(s: &crate::ast::TypeDefinition) -> UnresolvedStructSymbol {
        assert_eq!(s.keyword(), "struct");
        assert_eq!(s.bind(), None, "Bindings is not allowed in the structs");

        let identifier = s.name().to_string();
        let fields = s.fields().iter().map(UnresolvedStructField::new).collect::<Vec<_>>();
        let mut metadata = HashMap::new();

        s.attributes().iter().for_each(|a| {
            metadata.insert(a.name().to_string(), a.content().clone());
        });

        UnresolvedStructSymbol {
            identifier,
            fields,
            metadata,
            resolved: vec![],
        }
    }
}

impl TypeResolver<StructSymbol> for UnresolvedStructSymbol {
    fn resolve(&mut self, workspace: &Workspace) -> bool {
        self.fields.retain(|f| {
            if let Some(ty) = workspace.get_type(f.ty.namespace.as_deref(), &f.ty.identifier) {
                self.resolved.push(ResolvedField {
                    identifier: f.identifier.clone(),
                    ty
                });
                return false;
            }

            true
        });

        self.fields.is_empty()
    }

    fn as_resolved_type(&self) -> StructSymbol {
        assert!(self.fields.is_empty());
        StructSymbol {
            identifier: self.identifier.clone(),
            fields: self.resolved.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Symbol for StructSymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}
