use crate::ast::{BaseType, Count, Group};
use crate::{
    AnalysisWorkspace, SchemaModel, UnresolvedType,
    semantic::{
        TypeResolver,
        symbol::{Symbol, SymbolKind, SymbolRef},
    },
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GroupSymbol {
    identifier: String,
    extend: bool,
    groups: Vec<GroupConfig>,
}

impl GroupSymbol {
    #[must_use]
    pub fn groups(&self) -> &[GroupConfig] {
        &self.groups
    }

    #[must_use]
    pub fn extend(&self) -> bool {
        self.extend
    }

    #[must_use]
    pub fn main(root: SymbolRef) -> Self {
        Self {
            identifier: String::from("Main"),
            extend: false,
            groups: vec![GroupConfig {
                symbol: root,
                unique: true,
                count: Some(Count::Single(1)),
            }],
        }
    }

    #[must_use]
    pub fn get_constraints(&self) -> HashMap<SymbolRef, Count> {
        let mut map = HashMap::new();
        self.groups.iter().for_each(|g| {
            map.insert(g.symbol, g.count.unwrap_or(Count::ZeroOrMore)); //TODO set this value by default
        });
        map
    }

    #[must_use]
    pub fn get_unique_groups(&self) -> HashSet<SymbolRef> {
        let mut set = HashSet::new();
        self.groups.iter().filter(|g| g.unique).for_each(|g| {
            set.insert(g.symbol);
        });
        set
    }
}

#[derive(Debug, Clone)]
pub struct GroupConfig {
    symbol: SymbolRef,
    unique: bool,
    count: Option<Count>,
}

impl GroupConfig {
    #[must_use]
    pub fn symbol(&self) -> SymbolRef {
        self.symbol
    }
}

#[derive(Debug)]
pub struct UnresolvedGroupConfig {
    symbol: UnresolvedType,
    unique: bool,
    count: Option<Count>,
}

#[derive(Debug)]
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
        let mut metadata = HashMap::new();

        g.attributes.iter().for_each(|a| {
            metadata.insert(a.name.clone(), a.value.clone());
        });

        let extend = g.annotations.try_take("extend").is_some();

        let unresolved = g
            .entries
            .iter()
            .map(|g| {
                let identifier = g.name.to_string();
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

    pub const fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl TypeResolver<GroupSymbol> for UnresolvedGroupSymbol {
    fn resolve(&mut self, workspace: &mut AnalysisWorkspace) -> Result<bool, crate::Error> {
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

        Ok(self.unresolved.is_empty())
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

    fn try_get_self_reference(&self, model: &SchemaModel) -> Option<&SymbolRef> {
        for group in &self.groups {
            let ty = model.get_type_by_ref(group.symbol()).unwrap().expect("Unreachable!");
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
