use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Property {
    name: String,
    ty: String,
}

#[derive(Debug, Deserialize)]
pub struct Element {
    group: String,
    properties: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct Group {
    pub groups: Vec<String>,
    pub max: Option<u32>,
    pub min: u32,
}

//pub fn element_exist(&self, element: &str) -> bool {
//    self.elements.iter().any(|item| item.name.eq(element))
//}

#[derive(Clone, Debug, Deserialize)]
pub enum RawSchemaEntry {
    Namespace(String),
    Include(Vec<String>),
    Group {
        name: String,
        #[serde(default)]
        groups: Vec<String>,
        #[serde(default)]
        max: Option<u32>,
        #[serde(default)]
        min: u32,
        #[serde(default)]
        extend: bool,
    },
    Element {
        name: String,
        group: String,
        #[serde(default)]
        properties: BTreeMap<String, String>,
    },
}

#[derive(Debug)]
pub struct Schema {
    pub groups: HashMap<String, Group>,
    pub elements: HashMap<String, Element>,
}

impl Schema {
    pub fn from_scheme_configs(contents: &[&str]) -> Self {
        let mut extendable_groups = HashMap::new();
        let mut groups = HashMap::new();
        let mut elements = HashMap::new();

        // namespace -> entries
        let mut ns_map: HashMap<String, Vec<RawSchemaEntry>> = HashMap::new();
        let mut anon_entries = Vec::new();

        // 1. Разбираем все файлы
        contents.iter().for_each(|content| {
            let entries: Vec<RawSchemaEntry> = ron::from_str(content).expect("Invalid RON schema");

            let mut current_ns: Option<String> = None;
            let mut local_entries = Vec::new();

            entries.into_iter().for_each(|entry| match entry {
                RawSchemaEntry::Namespace(ns) => current_ns = Some(ns.clone()),
                _ => local_entries.push(entry),
            });

            if let Some(ns) = current_ns {
                ns_map.entry(ns).or_default().extend(local_entries);
            } else {
                anon_entries.extend(local_entries);
            }
        });

        // 2. Функция для полного разрешения includes
        fn resolve_entries(
            ns: &str,
            ns_map: &HashMap<String, Vec<RawSchemaEntry>>,
            visited: &mut Vec<String>,
        ) -> Vec<RawSchemaEntry> {
            if visited.iter().any(|item| item == ns) {
                return vec![]; // цикл
            }

            visited.push(ns.to_string());

            let mut result = Vec::new();
            if let Some(entries) = ns_map.get(ns) {
                for e in entries {
                    match e {
                        RawSchemaEntry::Include(includes) => {
                            for inc in includes {
                                let sub = resolve_entries(inc, ns_map, visited);
                                result.extend(sub);
                            }
                        }
                        _ => result.push(e.clone()),
                    }
                }
            }
            result
        }

        // 3. Собираем groups + elements
        ns_map.keys().for_each(|ns| {
            let resolved = resolve_entries(ns, &ns_map, &mut vec![]);
            resolved.into_iter().for_each(|entry| match entry {
                RawSchemaEntry::Group {
                    name,
                    groups: g,
                    max,
                    min,
                    extend,
                } => {
                    let fq_name = format!("{ns}:{name}");
                    let group = Group {
                        groups: g,
                        max,
                        min,
                    };

                    if extend {
                        extendable_groups.insert(fq_name, group);
                    } else {
                        groups.insert(fq_name, group);
                    }
                }
                RawSchemaEntry::Element {
                    name,
                    group,
                    properties,
                } => {
                    let fq_name = format!("{ns}:{name}");

                    if groups.contains_key(&group) {
                        panic!(
                            "Group '{group}' does not allow extension (extend = false), \
                                 but element '{fq_name}' tries to join it"
                        );
                    }

                    elements.insert(fq_name, Element { group, properties });
                }
                _ => {}
            });
        });

        // 4. Анонимные записи (без namespace)
        anon_entries.into_iter().for_each(|entry| match entry {
            RawSchemaEntry::Group {
                name,
                groups: g,
                max,
                min,
                extend,
            } => {
                let group = Group {
                    groups: g,
                    max,
                    min,
                };
                if extend {
                    extendable_groups.insert(name, group);
                } else {
                    groups.insert(name, group);
                }
            }
            RawSchemaEntry::Element {
                name,
                group,
                properties,
            } => {
                if groups.contains_key(&group) {
                    panic!(
                        "Group '{group}' does not allow extension (extend = false), \
                             but element '{name}' tries to join it"
                    );
                }

                elements.insert(name, Element { group, properties });
            }
            _ => {}
        });

        groups.extend(extendable_groups);

        Schema { groups, elements }
    }

    pub fn build_fsm(&self) {
        while !self.groups.is_empty() {
            self.groups.iter().for_each(|(name, group)| {});
        }
    }
}

pub struct State {
    symbols: HashMap<String, ()>,
}

pub struct RmlParser {
    start: String,
}

pub enum StateType {
    Attribute,
    Element,
}

#[cfg(test)]
mod tests {
    use crate::schema::Schema;

    const FILE_DB: &str = r#"
[
    Namespace("database"),
    Group(
        name: "userdata",
        groups: ["identifier", "message"],
    ),
    Group(
        name: "identifier",
        min: 1,
        max: Some(1),
        extend: true,
    ),
    Group(
        name: "message",
        min: 0,
        max: Some(1),
        extend: true,
    ),

    Element(
        name: "root",
        group: "userdata"
    )
]
"#;

    const FILE_USER: &str = r#"
[
    Include(["database"]),
    Element(
        name: "username",
        group: "database:identifier",
    ),
    Element(
        name: "user_message",
        group: "database:message",
    ),
]
"#;

    #[test]
    fn correct() {
        let schema = Schema::from_scheme_configs(&[FILE_DB, FILE_USER]);
        assert_eq!(schema.elements.len(), 3);
        assert_eq!(schema.groups.len(), 3);
    }

    #[test]
    fn graph() {
        let schema = Schema::from_scheme_configs(&[FILE_DB, FILE_USER]);
        let _graph = schema.build_fsm();
        println!();
    }
}
