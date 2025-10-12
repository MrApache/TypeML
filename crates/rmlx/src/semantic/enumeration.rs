use crate::{
    semantic::symbol::{Symbol, TypeRef},
    BaseType, Enum, TypeResolver, UnresolvedType, AnalysisWorkspace,
};
use std::collections::HashMap;

pub struct UnresolvedEnumSymbol {
    identifier: String,
    variants: Vec<UnresolvedVariant>,
    metadata: HashMap<String, Option<BaseType>>,
    resolved: Vec<EnumVariant>,
}

pub struct UnresolvedVariant {
    identifier: String,
    ty: Option<UnresolvedType>,
    pattern: Option<String>,
}

impl UnresolvedEnumSymbol {
    pub fn new(e: &Enum) -> Self {
        let identifier = e.name.clone();
        let mut variants = vec![];
        let mut metadata = HashMap::new();

        e.variants.iter().for_each(|v| {
            let identifier = v.name.clone();
            let ty = v.value.as_ref().map(|ty| ty.clone().into());

            //TODO annotations
            variants.push(UnresolvedVariant {
                identifier,
                ty,
                pattern: None,
                //pattern: v.pattern().map(str::to_string),
            });
        });

        e.attributes.iter().for_each(|a| {
            metadata.insert(a.name.to_string(), a.value.clone());
        });

        Self {
            identifier,
            variants,
            metadata,
            resolved: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumSymbol {
    pub identifier: String,
    pub variants: Vec<EnumVariant>,
    pub metadata: HashMap<String, Option<BaseType>>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub identifier: String,
    pub ty: Option<TypeRef>,
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

    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> bool {
        self.variants.retain(|v| {
            if let Some(ty) = &v.ty {
                if let Some(ty) = workspace.get_type(ty) {
                    self.resolved.push(EnumVariant {
                        identifier: v.identifier.clone(),
                        ty: Some(TypeRef::Concrete(ty)),
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
