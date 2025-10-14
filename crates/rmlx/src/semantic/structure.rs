use crate::ast::TypeIdent;
use crate::ast::TypeRef;
use crate::ast::{BaseType, Field, Struct};
use crate::{
    AnalysisWorkspace, SchemaModel, TypeResolver, UnresolvedType,
    semantic::symbol::{Symbol, SymbolRef},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StructSymbol {
    pub identifier: String,
    pub fields: Vec<ResolvedField>,
    pub metadata: HashMap<String, Option<BaseType>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedField {
    identifier: String,
    ty: SymbolRef,
}

pub struct UnresolvedStructField {
    identifier: String,
    ty: UnresolvedType,
}

impl From<TypeRef> for UnresolvedType {
    fn from(value: TypeRef) -> Self {
        match value.ident {
            TypeIdent::Simple(ident) => Self {
                generic_base: None,
                namespace: value.namespace,
                identifier: ident,
            },
            TypeIdent::Generic(ident, inner) => Self {
                generic_base: Some(ident),
                namespace: value.namespace,
                identifier: inner.to_string(), //TODO
            },
        }
    }
}

impl UnresolvedStructField {
    pub fn new(f: &Field) -> UnresolvedStructField {
        let identifier = f.name.to_string();
        let ty = f.ty.clone().into();
        UnresolvedStructField { identifier, ty }
    }
}

pub struct UnresolvedStructSymbol {
    pub identifier: String,
    pub fields: Vec<UnresolvedStructField>,
    pub metadata: HashMap<String, Option<BaseType>>,
    pub resolved: Vec<ResolvedField>,
}

impl UnresolvedStructSymbol {
    pub fn new(s: &Struct) -> UnresolvedStructSymbol {
        let identifier = s.name.clone();
        let fields = s.fields.iter().map(UnresolvedStructField::new).collect::<Vec<_>>();
        let mut metadata = HashMap::new();

        s.attributes.iter().for_each(|a| {
            metadata.insert(a.name.clone(), a.value.clone());
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

    fn can_parse(&self, value: &str, model: &SchemaModel) -> Result<bool, crate::Error> {
        let mut result = true;
        value.split(',').try_for_each(|field| {
            let mut parts = field.split('=');
            let name = parts.next().expect("Unreachable!").trim();
            let value = parts.next().expect("Unreachable!").trim();
            if let Some(field) = self.field(name) {
                let ty = model.get_type_by_ref(field.ty).unwrap().expect("Unreachable!");
                result &= ty.can_parse(value, model)?;
            } else {
                result = false;
            }

            Ok::<(), crate::Error>(())
        })?;

        Ok(result)
    }
}

impl StructSymbol {
    pub fn field(&self, name: &str) -> Option<&ResolvedField> {
        self.fields.iter().find(|f| f.identifier == name)
    }
}
