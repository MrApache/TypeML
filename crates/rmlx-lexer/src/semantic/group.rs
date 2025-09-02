use crate::{
    semantic::TypeResolver, AttributeNode, GroupNode, Namespace, RefNode, SymbolRef, TypeTable,
    Workspace,
};

pub struct Group {
    id: usize,
    name: String,
    config: GroupConfig,
    groups: Vec<SymbolRef>,
}

#[derive(Default)]
pub struct GroupConfig {
    min: Option<u32>,
    max: Option<u32>,
    extend: bool,
}

pub struct UnresolvedGroup {
    id: usize,
    name: String,
    config: GroupConfig,
    namespace: Namespace,
    resolved_groups: Vec<SymbolRef>,
    unresolved_groups: Vec<RefNode>,
}

impl TypeResolver for UnresolvedGroup {
    type Resolved = Group;

    fn try_resolve(&mut self, table: &TypeTable) {
        self.unresolved_groups.retain(|r| {
            let namespace = if let Some(ns) = &r.namespace {
                Namespace::Custom(ns.clone())
            } else {
                Namespace::Global
            };

            if let Some(ref_type) = table.get_type(&namespace, &r.identifier) {
                self.resolved_groups.push(ref_type.clone());
                return false;
            }

            true
        });
    }

    fn is_resolved(&self) -> bool {
        self.unresolved_groups.is_empty()
    }

    fn to_resolved_type(self) -> Self::Resolved {
        debug_assert!(self.unresolved_groups.is_empty());
        Self::Resolved {
            id: self.id,
            name: self.name,
            config: self.config,
            groups: self.resolved_groups,
        }
    }
}

impl Workspace {
    pub fn new_group(&mut self, id: usize, namespace: Namespace, node: &GroupNode) -> UnresolvedGroup {
        let config = resolve_group_attributes(&node.attributes);
        UnresolvedGroup {
            id,
            name: node.name.clone(),
            config,
            resolved_groups: vec![],
            unresolved_groups: node.groups.clone(),
            namespace,
        }
    }
/*
    fn resolve_ref_groups(&self, refs: &[RefNode]) -> Vec<SymbolRef> {
        let mut groups = vec![];
        refs.iter().for_each(|r| {
            let namespace = if let Some(ns) = &r.namespace {
                Namespace::Custom(ns.clone())
            } else {
                Namespace::Global
            };

            if let Some(types) = self.type_table.get(&namespace) {
                if let Some(ref_type) = types.get(&r.identifier) {
                    groups.push(ref_type.clone());
                } else {
                    //TODO report diagnostic 'type 'a' is not defined in the namespace 'b'
                }
            } else {
                //TODO report diagnostic 'the namespace 'a' is not declared'
            }
        });

        groups
    }
*/
}

fn resolve_group_attributes(attributes: &[AttributeNode]) -> GroupConfig {
    let mut config = GroupConfig::default();

    let mut seen_min = false;
    let mut seen_max = false;
    let mut seen_extend = false;

    for attr in attributes {
        match attr.name.as_str() {
            "Min" => {
                if seen_min {
                    // TODO report diagnostic: duplicate Min
                }
                seen_min = true;

                match &attr.value {
                    Some(v) => match v.parse::<u32>() {
                        Ok(num) => config.min = Some(num),
                        Err(_) => {
                            // TODO report diagnostic: invalid number
                        }
                    },
                    None => {
                        // TODO report diagnostic: Min requires value
                    }
                }
            }
            "Max" => {
                if seen_max {
                    // TODO report diagnostic: duplicate Max
                }
                seen_max = true;

                match &attr.value {
                    Some(v) => match v.parse::<u32>() {
                        Ok(num) => config.max = Some(num),
                        Err(_) => {
                            // TODO report diagnostic: invalid number
                        }
                    },
                    None => {
                        // TODO report diagnostic: Max requires value
                    }
                }
            }
            "Extend" => {
                if seen_extend {
                    // TODO report diagnostic: duplicate Extend
                }
                seen_extend = true;

                if attr.value.is_some() {
                    // TODO report diagnostic: Extend must not have a value
                } else {
                    config.extend = true;
                }
            }
            _ => {
                // TODO report diagnostic: unknown attribute
            }
        }
    }

    config
}
