use crate::ast::{AnnotationValue, BaseType, Enum};
use crate::{
    AnalysisWorkspace, Error, SchemaModel, TypeResolver, UnresolvedType,
    semantic::symbol::{Symbol, TypeRef},
};
use regex::Regex;
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
            let pattern = v.annotations.try_take("pattern");
            let pattern = if let Some(annotation) = pattern {
                if let Some(value) = annotation.value {
                    match value {
                        AnnotationValue::String(string) => Some(string),
                        AnnotationValue::Array(_) => panic!("The value should be a string"),
                    }
                } else {
                    None
                }
            } else {
                None
            };

            //TODO Annotations
            variants.push(UnresolvedVariant {
                identifier,
                ty,
                pattern,
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

    fn as_resolved_type(&self) -> EnumSymbol {
        assert!(self.variants.is_empty());
        EnumSymbol {
            identifier: self.identifier.clone(),
            variants: self.resolved.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Symbol for EnumSymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn can_parse(&self, value: &str, model: &SchemaModel) -> Result<bool, Error> {
        let default_inner_regex = Regex::new(r"([a-zA-Z][a-zA-Z0-9_]*)\((.*)\)")?;

        self.variants.iter().try_fold(false, |acc, v| {
            let mut result = false;

            if let Some(pattern) = &v.pattern {
                let regex = Regex::new(pattern)?;
                result = regex.is_match(value);
            }

            if !result
                && default_inner_regex.is_match(value)
                && let Some(ty) = &v.ty
                && let Some(cap) = default_inner_regex.captures(value)
            {
                let value = cap.get(2).expect("Unreachable!").as_str();
                let ty = model.get_type_by_ref(ty.as_concrete()).unwrap().expect("Unreachable!");
                result = v.identifier == cap.get(1).expect("Unreachable!").as_str() && ty.can_parse(value, model)?;
            }

            if !result {
                result = v.identifier == value;
            }

            Ok(acc || result)
        })
    }
}
