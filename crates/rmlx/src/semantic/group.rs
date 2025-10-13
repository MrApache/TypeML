use crate::{semantic::{
    symbol::{Symbol, SymbolKind, SymbolRef},
    TypeResolver,
}, AnalysisWorkspace, BaseType, Count, Group, SchemaModel, UnresolvedType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GroupSymbol {
    identifier: String,
    extend: bool,
    groups: Vec<GroupConfig>,
}

impl GroupSymbol {
    pub fn groups(&self) -> &[GroupConfig] {
        &self.groups
    }
}

#[derive(Debug, Clone)]
pub struct GroupConfig {
    symbol: SymbolRef,
    unique: bool,
    count: Option<Count>,
}

impl GroupConfig {
    pub fn symbol(&self) -> &SymbolRef {
        &self.symbol
    }
}

pub struct UnresolvedGroupConfig {
    symbol: UnresolvedType,
    unique: bool,
    count: Option<Count>,
}

pub struct UnresolvedGroupSymbol {
    identifier: String,
    extend: bool,
    unresolved: Vec<UnresolvedGroupConfig>,
    resolved: Vec<GroupConfig>,
    metadata: HashMap<String, Option<BaseType>>,
}

impl UnresolvedGroupSymbol {
    pub fn new(g: &Group) -> Self {
        let identifier = g.name.clone();
        let extend = g.extend;
        let mut metadata = HashMap::new();

        g.attributes.iter().for_each(|a| {
            metadata.insert(a.name.clone(), a.value.clone());
        });

        let unresolved = g
            .entries
            .iter()
            .map(|g| {
                let identifier = g.name.to_string();
                let unique = g.unique;
                UnresolvedGroupConfig {
                    symbol: UnresolvedType {
                        generic_base: None,
                        namespace: None,
                        identifier,
                    },
                    unique: g.unique,
                    count: g.count,
                }
            })
            .collect::<Vec<_>>();

        UnresolvedGroupSymbol {
            identifier,
            extend,
            metadata,
            unresolved,
            resolved: vec![],
        }
    }
}

impl TypeResolver<GroupSymbol> for UnresolvedGroupSymbol {
    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> bool {
        self.unresolved.retain(|f| {
            if f.symbol.identifier == self.identifier {
                let symbol = workspace.create_self_reference(&f.symbol);
                self.resolved.push(GroupConfig {
                    symbol,
                    unique: f.unique,
                    count: f.count,
                });
                return false;
            } else if let Some(symbol) = workspace.get_type(&f.symbol) {
                self.resolved.push(GroupConfig {
                    symbol,
                    unique: f.unique,
                    count: f.count,
                });
                return false;
            }

            true
        });

        self.unresolved.is_empty()
    }

    fn as_resolved_type(&self) -> GroupSymbol {
        GroupSymbol {
            identifier: self.identifier.clone(),
            extend: self.extend,
            groups: self.resolved.clone(),
        }
    }
}

impl Symbol for GroupSymbol {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn can_parse(&self, value: &str, model: &SchemaModel) -> bool {
        false
    }

    fn try_get_self_reference(&self, model: &SchemaModel) -> Option<&SymbolRef> {
        for group in &self.groups {
            let ty = model.get_type_by_id(group.symbol.namespace.as_deref(), group.symbol.id).unwrap();
            match ty {
                SymbolKind::Lazy(lazy) => {
                    if lazy.identifier == self.identifier {
                        return Some(&group.symbol);
                    }
                }
                _ => return None,
            }
        }
        None
    }
}
