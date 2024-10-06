use std::any::TypeId;

use bevy::{prelude::*, utils::Entry};

use super::{
    input_action::{Accumulation, ActionData, ActionsData, InputAction},
    input_condition::InputCondition,
    input_modifier::InputModifier,
    trigger_tracker::TriggerTracker,
};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input_reader::{Input, InputReader},
    prelude::{Negate, SwizzleAxis},
};

#[derive(Default)]
pub struct ContextMap {
    priority: usize,
    actions: Vec<ActionMap>,
    actions_data: ActionsData,
}

impl ContextMap {
    pub fn with_priority(priority: usize) -> Self {
        Self {
            priority,
            ..Default::default()
        }
    }

    pub fn bind<A: InputAction>(&mut self) -> &mut ActionMap {
        let type_id = TypeId::of::<A>();
        match self.actions_data.entry(type_id) {
            Entry::Occupied(_entry) => self
                .actions
                .iter_mut()
                .find(|action_map| action_map.type_id == type_id)
                .expect("data and actions should have matching type IDs"),
            Entry::Vacant(entry) => {
                entry.insert(ActionData::new::<A>());
                self.actions.push(ActionMap::new::<A>());
                self.actions.last_mut().unwrap()
            }
        }
    }

    pub(super) fn update(
        &mut self,
        world: &World,
        commands: &mut Commands,
        reader: &mut InputReader,
        entity: Entity,
        delta: f32,
    ) {
        for action_map in &mut self.actions {
            action_map.update(
                world,
                commands,
                reader,
                &mut self.actions_data,
                entity,
                delta,
            );
        }
    }

    pub(super) fn trigger_removed(mut self, commands: &mut Commands, entity: Entity) {
        // TODO: Consider redundantly store dimention in the data.
        for action_map in self.actions.drain(..) {
            let data = self
                .actions_data
                .remove(&action_map.type_id)
                .expect("data and actions should have matching type IDs");
            data.trigger_removed(commands, entity, action_map.dim);
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ActionMap> {
        self.actions.iter_mut()
    }

    pub(super) fn priority(&self) -> usize {
        self.priority
    }
}

pub struct ActionMap {
    type_id: TypeId,
    consumes_input: bool,
    accumulation: Accumulation,
    dim: ActionValueDim,
    last_value: ActionValue,

    modifiers: Vec<Box<dyn InputModifier>>,
    conditions: Vec<Box<dyn InputCondition>>,
    inputs: Vec<InputMap>,
}

impl ActionMap {
    fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            dim: A::DIM,
            consumes_input: A::CONSUMES_INPUT,
            accumulation: A::ACCUMULATION,
            last_value: ActionValue::zero(A::DIM),
            modifiers: Default::default(),
            conditions: Default::default(),
            inputs: Default::default(),
        }
    }

    pub fn with_wasd(&mut self) -> &mut Self {
        self.with(InputMap::new(KeyCode::KeyW).with_modifier(SwizzleAxis::YXZ))
            .with(InputMap::new(KeyCode::KeyA).with_modifier(Negate))
            .with(
                InputMap::new(KeyCode::KeyS)
                    .with_modifier(Negate)
                    .with_modifier(SwizzleAxis::YXZ),
            )
            .with(InputMap::new(KeyCode::KeyD))
    }

    pub fn with_modifier(&mut self, modifier: impl InputModifier) -> &mut Self {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn with_condition(&mut self, condition: impl InputCondition) -> &mut Self {
        self.conditions.push(Box::new(condition));
        self
    }

    pub fn with(&mut self, map: impl Into<InputMap>) -> &mut Self {
        self.inputs.push(map.into());
        self
    }

    pub fn clear_mappings(&mut self) {
        self.inputs.clear();
    }

    fn update(
        &mut self,
        world: &World,
        commands: &mut Commands,
        reader: &mut InputReader,
        actions_data: &mut ActionsData,
        entity: Entity,
        delta: f32,
    ) {
        let mut tracker = TriggerTracker::new(ActionValue::zero(self.dim));
        for input_map in &mut self.inputs {
            if let Some(value) = reader.read(input_map.input, self.consumes_input) {
                self.last_value = value.convert(self.dim);
            }
            let mut current_tracker = TriggerTracker::new(self.last_value);
            current_tracker.apply_modifiers(world, delta, &mut input_map.modifiers);
            current_tracker.apply_conditions(world, actions_data, delta, &mut input_map.conditions);
            tracker.merge(current_tracker, self.accumulation);
        }

        tracker.apply_modifiers(world, delta, &mut self.modifiers);
        tracker.apply_conditions(world, actions_data, delta, &mut self.conditions);

        let (state, value) = tracker.finish();
        let data = actions_data
            .get_mut(&self.type_id)
            .expect("data and actions should have matching type IDs");

        data.update(commands, entity, state, value, delta);
    }
}

pub struct InputMap {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,
}

impl InputMap {
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            modifiers: Default::default(),
            conditions: Default::default(),
        }
    }

    pub fn with_modifier(mut self, modifier: impl InputModifier) -> Self {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn with_condition(mut self, condition: impl InputCondition) -> Self {
        self.conditions.push(Box::new(condition));
        self
    }
}

impl From<KeyCode> for InputMap {
    fn from(value: KeyCode) -> Self {
        Self::new(value)
    }
}
