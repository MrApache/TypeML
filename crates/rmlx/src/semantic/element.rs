use std::collections::HashMap;
use crate::{semantic::symbol::{Symbol, SymbolRef}, TypeResolver, UnresolvedType, Workspace};

pub struct ElementSymbol {
    identifier: String,
    fields: Vec<ResolvedField>,
    bind: SymbolRef,
    metadata: HashMap<String, Option<String>>,
}

#[derive(Clone)]
pub struct ResolvedField {
    identifier: String,
    ty: SymbolRef,
}

pub struct UnresolvedElementField {
    identifier: String,
    ty: crate::ast::Type,
}

impl UnresolvedElementField {
    pub fn new(f: &crate::ast::Field) -> UnresolvedElementField {
        let identifier = f.name().to_string();
        let ty = f.ty().as_simple_or_generic();
        UnresolvedElementField {
            identifier,
            ty
        }
    }
}

pub struct UnresolvedElementSymbol {
    pub identifier: String,
    pub bind: UnresolvedType,
    pub resolved_bind: Option<SymbolRef>,
    pub fields: Vec<UnresolvedElementField>,
    pub metadata: HashMap<String, Option<String>>,
    pub resolved: Vec<ResolvedField>,
}

impl UnresolvedElementSymbol {
    pub fn new(s: &crate::ast::TypeDefinition) -> UnresolvedElementSymbol {
        assert_eq!(s.keyword(), "element");
        assert_ne!(s.bind(), None, "Missing binding specifier");

        let identifier = s.name().to_string();
        let bind = s.bind().unwrap();
        let bind = UnresolvedType {
            namespace: None,
            identifier: bind.name().to_string(),
        };
        let fields = s.fields().iter().map(UnresolvedElementField::new).collect::<Vec<_>>();
        let mut metadata = HashMap::new();

        s.attributes().iter().for_each(|a| {
            metadata.insert(a.name().to_string(), a.content().clone());
        });

        UnresolvedElementSymbol {
            identifier,
            bind,
            resolved_bind: None,
            fields,
            metadata,
            resolved: vec![],
        }
    }
}

impl TypeResolver<ElementSymbol> for UnresolvedElementSymbol {
    fn resolve(&mut self, workspace: &Workspace) -> bool {
        self.fields.retain(|f| {
            match &f.ty {
                crate::Type::Simple(value) => {
                    if let Some(ty) = workspace.get_type(None, &value) {
                        self.resolved.push(ResolvedField {
                            identifier: f.identifier.clone(),
                            ty
                        });
                        return false;
                    }
                },
                crate::Type::Generic(base, value) => {
                    if let Some(ty) = workspace.get_type(None, &value) {
                        self.resolved.push(ResolvedField {
                            identifier: f.identifier.clone(),
                            ty
                        });
                        return false;
                    }
                },
                _ => unreachable!(),
            }

            true
        });

        if self.resolved_bind.is_none() {
            if let Some(ty) = workspace.get_type(self.bind.namespace.as_deref(), &self.bind.identifier) {
                self.resolved_bind = Some(ty);
            }
        }

        self.fields.is_empty() && self.resolved_bind.is_some()
    }

    fn as_resolved_type(&self) -> ElementSymbol {
        assert!(self.fields.is_empty());
        assert!(self.resolved_bind.is_some());
        ElementSymbol {
            identifier: self.identifier.clone(),
            fields: self.resolved.clone(),
            metadata: self.metadata.clone(),
            bind: self.resolved_bind.clone().unwrap(),
        }
    }
}

impl Symbol for ElementSymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}
