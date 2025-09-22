use crate::{
    semantic::symbol::{Symbol, SymbolRef},
    TypeResolver, UnresolvedType,
};
use std::collections::HashMap;

pub struct UnresolvedEnumSymbol {
    identifier: String,
    variants: Vec<UnresolvedVariant>,
    metadata: HashMap<String, Option<String>>,
    resolved: Vec<EnumVariant>,
}

pub struct UnresolvedVariant {
    identifier: String,
    ty: Option<UnresolvedType>,
    pattern: Option<String>,
}

impl UnresolvedEnumSymbol {
    pub fn new(e: &crate::ast::Enum) -> Self {
        let identifier = e.name().to_string();
        let mut variants = vec![];
        let mut metadata = HashMap::new();

        e.variants().iter().for_each(|v| {
            let identifier = v.name().to_string();
            let ty = if let Some(ty) = v.ty() {
                Some(UnresolvedType {
                    namespace: None,
                    identifier: ty.to_string(),
                })
            } else {
                None
            };

            variants.push(UnresolvedVariant {
                identifier,
                ty,
                pattern: v.pattern().map(str::to_string),
            });
        });

        e.attributes().iter().for_each(|a| {
            metadata.insert(a.name().to_string(), a.content().clone());
        });

        Self {
            identifier,
            variants,
            metadata,
            resolved: vec![],
        }
    }
}

pub struct EnumSymbol {
    pub identifier: String,
    pub variants: Vec<EnumVariant>,
    pub metadata: HashMap<String, Option<String>>,
}

#[derive(Clone)]
pub struct EnumVariant {
    pub identifier: String,
    pub ty: Option<SymbolRef>,
    pub pattern: Option<String>,
}

impl TypeResolver<EnumSymbol> for UnresolvedEnumSymbol {
    fn as_resolved_type(&self) -> EnumSymbol {
        assert!(self.variants.is_empty());
        EnumSymbol {
            identifier: self.identifier.clone(),
            variants: self.resolved.clone(),
            metadata: self.metadata.clone(),
        }
    }

    fn resolve(&mut self, workspace: &super::Workspace) -> bool {
        self.variants.retain(|v| {
            if let Some(ty) = &v.ty {
                if let Some(ty) = workspace.get_type(ty.namespace.as_deref(), &ty.identifier) {
                    self.resolved.push(EnumVariant {
                        identifier: v.identifier.clone(),
                        ty: Some(ty),
                        pattern: v.pattern.clone(),
                    });
                    return false;
                }
                return true;
            }
            self.resolved.push(EnumVariant {
                identifier: v.identifier.clone(),
                ty: None,
                pattern: v.pattern.clone(),
            });

            false
        });
        self.variants.is_empty()
    }
}

impl Symbol for EnumSymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}
