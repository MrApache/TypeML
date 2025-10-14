use crate::ast::{BaseType, Element, Field};
use crate::{
    AnalysisWorkspace, SchemaModel, TypeResolver, UnresolvedType,
    semantic::symbol::{Symbol, SymbolRef},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ElementSymbol {
    identifier: String,
    fields: Vec<ResolvedField>,
    bind: SymbolRef,
    metadata: HashMap<String, Option<BaseType>>,
}

impl ElementSymbol {
    pub fn group(&self) -> SymbolRef {
        self.bind
    }

    pub fn fields(&self) -> &[ResolvedField] {
        &self.fields
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedField {
    identifier: String,
    ty: SymbolRef,
}

impl ResolvedField {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn ty(&self) -> SymbolRef {
        self.ty
    }
}

pub struct UnresolvedElementField {
    identifier: String,
    ty: UnresolvedType,
}

impl UnresolvedElementField {
    pub fn new(f: &Field) -> UnresolvedElementField {
        let identifier = f.name.to_string();
        let ty = f.ty.clone().into();
        UnresolvedElementField { identifier, ty }
    }
}

pub struct UnresolvedElementSymbol {
    pub identifier: String,
    pub bind: UnresolvedType,
    pub resolved_bind: Option<SymbolRef>,
    pub fields: Vec<UnresolvedElementField>,
    pub metadata: HashMap<String, Option<BaseType>>,
    pub resolved: Vec<ResolvedField>,
}

impl UnresolvedElementSymbol {
    pub fn new(s: &Element) -> UnresolvedElementSymbol {
        let identifier = s.name.to_string();
        let bind = s.bind.clone().into();
        let fields = s.fields.iter().map(UnresolvedElementField::new).collect::<Vec<_>>();
        let mut metadata = HashMap::new();

        s.attributes.iter().for_each(|a| {
            metadata.insert(a.name.clone(), a.value.clone());
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
    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> bool {
        self.fields.retain(|f| {
            if let Some(ty) = workspace.get_type(&f.ty) {
                self.resolved.push(ResolvedField {
                    identifier: f.identifier.clone(),
                    ty,
                });
                return false;
            }
            true
        });

        if self.resolved_bind.is_none()
            && let Some(ty) = workspace.get_type(&self.bind)
        {
            self.resolved_bind = Some(ty);
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
            bind: self.resolved_bind.expect("Unreachable!"),
        }
    }
}

impl Symbol for ElementSymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn can_parse(&self, value: &str, model: &SchemaModel) -> Result<bool, crate::Error> {
        Ok(false)
    }
}

impl ElementSymbol {
    pub fn field(&self, name: &str) -> Option<&ResolvedField> {
        self.fields.iter().find(|f| f.identifier == name)
    }
}
