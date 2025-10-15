use crate::ast::{ArgumentValue, Element, Expression};
use rmlx::{Count, CountEquality, ExpressionField, ExpressionSymbol, Symbol};
use rmlx::{GroupConfig, SchemaModel, SymbolRef};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

pub struct AnalyzerState {
    group: SymbolRef,
    allowed: Vec<usize>,
}

pub struct PreviousElement {
    name: String,
    namespace: Option<String>,
    group: SymbolRef,
    state: usize,
    is_container: bool,
    uniques: HashSet<SymbolRef>,
    constraints: HashMap<SymbolRef, Count>,
    counter: HashMap<SymbolRef, HashMap<(Option<String>, String), u32>>,
}

pub struct RmlAnalyzer {
    model: Arc<RwLock<SchemaModel>>,
    depth: Vec<PreviousElement>,
    states: Vec<AnalyzerState>,
    active: usize,
}

impl RmlAnalyzer {
    fn build_states(root: SymbolRef, model: &SchemaModel) -> Vec<AnalyzerState> {
        let mut states = Vec::new();
        let mut visited = HashSet::new();
        let mut to_process = vec![root];

        // Build a mapping from SymbolRef to state index
        let mut state_indices = HashMap::new();

        while let Some(current_symbol) = to_process.pop() {
            if visited.contains(&current_symbol) {
                continue;
            }
            visited.insert(current_symbol);

            if let Some(ty) = model.get_type_by_ref(current_symbol).unwrap() {
                let group = ty.as_group_symbol();

                // Get all reachable groups from this state
                let reachable_groups: Vec<SymbolRef> = group.groups().iter().map(GroupConfig::symbol).collect();

                // Add new symbols to processing queue
                for symbol in reachable_groups {
                    if !visited.contains(&symbol) && !to_process.contains(&symbol) {
                        to_process.push(symbol);
                    }
                }
                // Create state with placeholder allowed indices (will be filled later)
                let state = AnalyzerState {
                    group: current_symbol,
                    allowed: Vec::new(), // will populate after all states are created
                };

                state_indices.insert(current_symbol, states.len());
                states.push(state);
            }
        }

        // Now populate the allowed transitions
        for state in &mut states {
            if let Some(ty) = model.get_type_by_ref(state.group).unwrap() {
                let group = ty.as_group_symbol();
                let reachable_groups: Vec<SymbolRef> = group.groups().iter().map(GroupConfig::symbol).collect();

                state.allowed = reachable_groups
                    .iter()
                    .filter_map(|symbol| state_indices.get(symbol))
                    .copied()
                    .collect();
            }
        }

        states
    }

    #[must_use]
    pub fn new(model: Arc<RwLock<SchemaModel>>) -> Self {
        let read_model = model.read().expect("Unreachable!");
        let group = read_model.get_main_group_ref();
        let states = Self::build_states(group, &read_model);
        drop(read_model);

        Self {
            model,
            depth: vec![],
            states,
            active: 0,
        }
    }

    pub fn is_allowed_element(&self, namespace: Option<&str>, name: &str) -> Result<bool, rmlx::Error> {
        let model = self.model.read().expect("Unreachable!");
        let namespace_id = model.get_namespace_id(namespace)?;
        let element = model
            .get_type_by_name(namespace_id, name)
            .as_element_symbol()
            .ok_or(rmlx::Error::ElementNotFound(name.into()))?;
        let bind_group = element.group();

        let group_ref = self.states[self.active].group;
        let ty = model.get_type_by_ref(group_ref).unwrap().expect("Unreachable!");
        let group = ty.as_group_symbol();
        let groups = group.groups();
        Ok(groups.iter().any(|g| g.symbol() == bind_group))
    }

    pub fn enter_element(&mut self, namespace: Option<&str>, name: &str) -> Result<(), rmlx::Error> {
        debug_assert!(self.is_allowed_element(namespace, name)?);

        let model = self.model.read().expect("Unreachable!");
        let namespace_id = model.get_namespace_id(namespace)?;
        let element = model
            .get_type_by_name(namespace_id, name)
            .as_element_symbol()
            .expect("Unreachable!");
        let bind_group = element.group();

        let ty = model.get_type_by_ref(bind_group).unwrap().expect("Unreachable!");
        let group = ty.as_group_symbol();
        if group.groups().is_empty() {
            self.depth.push(PreviousElement {
                name: name.to_string(),
                namespace: namespace.map(str::to_string),
                group: bind_group,
                state: self.active,
                is_container: false,
                counter: HashMap::default(),
                constraints: group.get_constraints(),
                uniques: group.get_unique_groups(),
            });
            return Ok(());
        }

        let active = &self.states[self.active];
        let next = *active
            .allowed
            .iter()
            .find(|allowed| {
                let state = &self.states[**allowed];
                state.group == bind_group
            })
            .expect("Unreachable!");

        self.depth.push(PreviousElement {
            name: name.to_string(),
            namespace: namespace.map(str::to_string),
            group: bind_group,
            state: self.active,
            is_container: true,
            counter: HashMap::default(),
            constraints: group.get_constraints(),
            uniques: group.get_unique_groups(),
        });
        self.active = next;

        Ok(())
    }

    fn get_group_full_path(&self, group: SymbolRef) -> String {
        let model = self.model.read().expect("Unreachable!");
        let group_kind = model.get_type_by_ref(group).unwrap().expect("Unreachable!");
        let namespace = &model.namespaces[group.namespace];
        format!("{namespace}::{}", group_kind.identifier())
    }

    fn check_and_change_counter(
        &mut self,
        group: SymbolRef,
        namespace: Option<&str>,
        name: &str,
    ) -> Result<(), rmlx::Error> {
        if let Some(last) = self.depth.last_mut() {
            let counter = last.counter.entry(group).or_default();
            let actual_count = counter
                .entry((namespace.map(str::to_string), name.to_string()))
                .or_default();
            *actual_count += 1;

            let actual_count = *actual_count;
            let count = *last.constraints.get(&group).unwrap();
            let result = count.in_range(actual_count);
            return match result {
                CountEquality::More => Err(rmlx::Error::ExcessiveElements {
                    group: self.get_group_full_path(group),
                    actual: actual_count,
                    expected: count,
                }),
                CountEquality::Less => Err(rmlx::Error::InsufficientElements {
                    group: self.get_group_full_path(group),
                    actual: actual_count,
                    expected: count,
                }),
                CountEquality::Ok => Ok(()),
            };
        }

        Ok(())
    }

    fn check_elements_uniqueness(&self) -> Result<(), rmlx::Error> {
        if let Some(last) = self.depth.last() {
            last.uniques.iter().try_for_each(|group| {
                if let Some(elements) = last.counter.get(group) {
                    if let Some(((ns, ident), count)) = elements.iter().find(|((_, _), count)| **count > 1) {
                        let full_path = format!("{}::{}", ns.clone().unwrap_or_default(), ident);
                        Err(rmlx::Error::NotUniqueElement(full_path))
                    } else {
                        Ok::<(), rmlx::Error>(())
                    }
                } else {
                    Ok::<(), rmlx::Error>(())
                }
            })?;
        }

        Ok(())
    }

    pub fn exit_element(&mut self, namespace: Option<&str>, name: &str) -> Result<(), rmlx::Error> {
        let previous_element = self.depth.pop().expect("Unreachable!");
        assert!(previous_element.name == name && previous_element.namespace.as_deref() == namespace);
        self.active = previous_element.state;
        self.check_and_change_counter(previous_element.group, namespace, name)?;
        self.check_elements_uniqueness()
    }

    pub fn is_valid_attribute(&self, name: &str, value: &str) -> Result<(), rmlx::Error> {
        let model = self.model.read().expect("Unreachable!");
        let last_element = self.depth.last().expect("Unreachable!");
        let element_namespace = model.get_namespace_id(last_element.namespace.as_deref())?;
        let element = model
            .get_type_by_name(element_namespace, &last_element.name)
            .as_element_symbol()
            .expect("Unreachable!");
        let field = element.field(name).expect("Unreachable!");
        let field_type = model.get_type_by_ref(field.ty());
        let field_type = field_type.as_ref();
        field_type.can_parse(value, &model)
    }

    fn is_valid_expression_element_group(
        namespace: Option<&str>,
        name: &str,
        groups: &[SymbolRef],
        model: &SchemaModel,
        expression: &Expression,
    ) -> Result<(), rmlx::Error> {
        let namespace_id = model.get_namespace_id(namespace)?;
        let element = model
            .get_type_by_name(namespace_id, name)
            .as_element_symbol()
            .expect("Unreachable!");

        let bind_group = element.group();
        if !groups.contains(&bind_group) {
            let ty = model.get_type_by_ref(bind_group).unwrap().expect("Unreachable!");
            let group = ty.as_group_symbol();
            return Err(rmlx::Error::ExpressionIsNotAllowedInGroup(
                expression.full_path(),
                group.identifier().to_string(),
            ));
        }

        Ok(())
    }

    fn validate_expression_fields(
        model: &SchemaModel,
        expr: &ExpressionSymbol,
        expression: &Expression,
    ) -> Result<(), rmlx::Error> {
        let field_map: HashMap<&str, &ExpressionField> =
            expr.fields().iter().map(|field| (field.identifier(), field)).collect();

        let mut used_fields = HashSet::new();

        // Validate all provided arguments
        for arg in &expression.arguments {
            let arg_identifier = arg.identifier.as_str();

            if let Some(field) = field_map.get(arg_identifier) {
                // Check for duplicate fields
                if !used_fields.insert(arg_identifier) {
                    return Err(rmlx::Error::DuplicateField(arg.identifier.to_string()));
                }

                // Validate field type
                let field_type = model.get_type_by_ref(field.ty());
                let field_type = field_type.as_ref();
                field_type.can_parse(arg.value.as_str(), model)?;
            } else {
                // Field doesn't exist in expression definition
                return Err(rmlx::Error::FieldNotFound(arg.identifier.to_string()));
            }
        }

        // Check for missing required fields
        for field in expr.fields() {
            if !field.is_optional() && !used_fields.contains(field.identifier()) {
                return Err(rmlx::Error::MissingRequiredField(field.identifier().to_string()));
            }
        }

        Ok(())
    }

    pub fn is_valid_expression(
        &self,
        element_namespace: Option<&str>,
        element_name: &str,
        expression: &Expression,
    ) -> Result<(), rmlx::Error> {
        let model = self.model.read().expect("Unreachable!");
        let expr_namespace = model.get_namespace_id(expression.namespace.as_deref())?;
        let expr = model
            .get_type_by_name(expr_namespace, &expression.identifier)
            .as_expression_symbol()
            .ok_or(rmlx::Error::ExpressionNotFound(expression.full_path()))?;

        Self::validate_expression_fields(&model, expr, expression)?;
        Self::is_valid_expression_element_group(element_namespace, element_name, expr.groups(), &model, expression)?;

        Ok(())
    }
}
