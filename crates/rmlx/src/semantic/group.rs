use std::collections::HashMap;

use crate::{
    semantic::{symbol::{Symbol, SymbolRef}, TypeResolver}, Count, UnresolvedType, Workspace
};

pub struct GroupSymbol {
    identifier: String,
    extend: bool,
    groups: Vec<GroupConfig>,
}

#[derive(Clone)]
pub struct GroupConfig {
    symbol: SymbolRef,
    unique: bool,
    count: Option<Count>,
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
}

impl UnresolvedGroupSymbol {
    pub fn new(g: &crate::ast::Group) -> Self {
        Self::new_group(g, false)
    }

    pub fn new_extendable(g: &crate::ast::Group) -> Self {
        Self::new_group(g, true)
    }

    fn new_group(g: &crate::ast::Group, extend: bool) -> Self {
        let identifier = g.name().to_string();
        let mut metadata = HashMap::new();

        g.attributes().iter().for_each(|a| {
            metadata.insert(a.name().to_string(), a.content().clone());
        });

        let unresolved = g.groups().iter().map(|g| {
            let identifier = g.name().to_string();
            let count = g.count().clone();
            let unique = g.unique();
            UnresolvedGroupConfig {
                symbol: UnresolvedType {
                    namespace: None,
                    identifier,
                },
                unique,
                count,
            }
        }).collect::<Vec<_>>();

        UnresolvedGroupSymbol {
            identifier,
            extend,
            unresolved,
            resolved: vec![],
        }
    }
}

impl TypeResolver<GroupSymbol> for UnresolvedGroupSymbol {
    fn resolve(&mut self, workspace: &Workspace) -> bool {
        self.unresolved.retain(|f| {
            if let Some(symbol) = workspace.get_type(f.symbol.namespace.as_deref(), &f.symbol.identifier) {
                self.resolved.push(GroupConfig {
                    symbol,
                    unique: f.unique,
                    count: f.count.clone(),
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
    fn identifier(&self) ->  &str {
        &self.identifier
    }
}
