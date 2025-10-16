use crate::ast::{Annotation, AnnotationValue, BaseType, Expression, Field};
use crate::{AnalysisWorkspace, Symbol, SymbolRef, TypeResolver, UnresolvedType};
use std::collections::HashMap;

#[derive(Debug)]
pub struct UnresolvedExpressionField {
    identifier: String,
    ty: UnresolvedType,
    optional: bool,
}

impl UnresolvedExpressionField {
    pub fn new(f: &Field) -> Self {
        let identifier = f.name.to_string();
        let ty = f.ty.clone().into();
        let optional = f.annotations.try_take("optional").is_some();
        Self {
            identifier,
            ty,
            optional,
        }
        // TODO Annotations
    }
}

#[derive(Debug)]
pub struct UnresolvedExpressionSymbol {
    identifier: String,
    metadata: HashMap<String, Option<BaseType>>,

    fields: Vec<UnresolvedExpressionField>,
    resolved_fields: Vec<ExpressionField>,

    groups: Vec<UnresolvedType>,
    resolved_groups: Vec<SymbolRef>,

    restrict: Vec<UnresolvedType>,
    resolved_restrict: Vec<SymbolRef>,
}

fn try_take_annotation(annotation: Option<Annotation>) -> Vec<UnresolvedType> {
    if let Some(annotation) = annotation
        && let Some(value) = annotation.value
    {
        match value {
            AnnotationValue::Array(array) => array
                .into_iter()
                .map(|group| {
                    UnresolvedType {
                        generic_base: None,
                        namespace: None, //TODO namespace
                        identifier: group,
                    }
                })
                .collect(),
            AnnotationValue::String(_) => panic!("Value should be an array"),
        }
    } else {
        vec![]
    }
}

impl UnresolvedExpressionSymbol {
    pub fn new(e: &Expression) -> Self {
        let identifier = e.name.to_string();
        let fields = e.fields.iter().map(UnresolvedExpressionField::new).collect::<Vec<_>>();
        let mut metadata = HashMap::new();

        e.attributes.iter().for_each(|a| {
            metadata.insert(a.name.clone(), a.value.clone());
        });

        let groups = try_take_annotation(e.annotations.try_take("groups"));
        let restrict = try_take_annotation(e.annotations.try_take("restrict"));

        Self {
            identifier,
            metadata,
            fields,
            groups,
            restrict,
            resolved_restrict: vec![],
            resolved_groups: vec![],
            resolved_fields: vec![],
        }
    }

    pub const fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl TypeResolver<ExpressionSymbol> for UnresolvedExpressionSymbol {
    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> Result<bool, crate::Error> {
        self.fields.retain(|f| {
            if let Some(ty) = workspace.get_type(&f.ty) {
                self.resolved_fields.push(ExpressionField {
                    identifier: f.identifier.clone(),
                    ty,
                    optional: f.optional,
                });
                return false;
            }
            true
        });

        self.groups.retain(|f| {
            if let Some(ty) = workspace.get_type(f) {
                self.resolved_groups.push(ty);
                return false;
            }
            true
        });

        self.restrict.retain(|f| {
            if let Some(ty) = workspace.get_type(f) {
                self.resolved_restrict.push(ty);
                return false;
            }
            true
        });

        Ok(self.fields.is_empty() && self.groups.is_empty() && self.restrict.is_empty())
    }

    fn as_resolved_type(&self) -> ExpressionSymbol {
        assert!(self.fields.is_empty());
        assert!(self.groups.is_empty());
        assert!(self.restrict.is_empty());
        ExpressionSymbol {
            identifier: self.identifier.clone(),
            metadata: self.metadata.clone(),
            fields: self.resolved_fields.clone(),
            groups: self.resolved_groups.clone(),
            restrict: self.resolved_restrict.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionField {
    identifier: String,
    ty: SymbolRef,
    optional: bool,
}

impl ExpressionField {
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    #[must_use]
    pub fn ty(&self) -> SymbolRef {
        self.ty
    }

    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.optional
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionSymbol {
    identifier: String,
    metadata: HashMap<String, Option<BaseType>>,
    fields: Vec<ExpressionField>,
    groups: Vec<SymbolRef>,
    restrict: Vec<SymbolRef>,
}

impl ExpressionSymbol {
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&ExpressionField> {
        self.fields.iter().find(|f| f.identifier == name)
    }

    #[must_use]
    pub fn fields(&self) -> &[ExpressionField] {
        &self.fields
    }

    #[must_use]
    pub fn groups(&self) -> &[SymbolRef] {
        &self.groups
    }

    #[must_use]
    pub fn restrict(&self) -> &[SymbolRef] {
        &self.restrict
    }
}

impl Symbol for ExpressionSymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }
}
