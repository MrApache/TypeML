use crate::ast::{ArgumentValue, Element, Expression};
use rmlx::Symbol;
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
    state: usize,
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

    pub fn next_state(&mut self, namespace: Option<&str>, name: &str) -> Result<(), rmlx::Error> {
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
                state: self.active,
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
            state: self.active,
        });
        self.active = next;

        Ok(())
    }

    pub fn exit_state(&mut self, namespace: Option<&str>, name: &str) {
        let previous_element = self.depth.pop().expect("Unreachable!");
        assert!(previous_element.name == name && previous_element.namespace.as_deref() == namespace);
        self.active = previous_element.state;
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

        expression.arguments.iter().try_for_each(|arg| {
            if let Some(field) = expr.field(&arg.identifier) {
                let field_type = model.get_type_by_ref(field.ty());
                let field_type = field_type.as_ref();
                field_type.can_parse(arg.value.as_str(), &model)
            } else {
                Err(rmlx::Error::FieldNotFound(arg.identifier.to_string()))
            }
        })?;

        Self::is_valid_expression_element_group(element_namespace, element_name, expr.groups(), &model, expression)?;

        Ok(())
    }
}
