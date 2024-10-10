use std::any::{self, TypeId};

use bevy::{prelude::*, utils::Entry};

use super::{
    input_action::{Accumulation, ActionData, ActionsData, InputAction},
    input_condition::InputCondition,
    input_modifier::InputModifier,
    trigger_tracker::TriggerTracker,
};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input_reader::{GamepadDevice, Input, InputReader},
    prelude::{Negate, SwizzleAxis},
};

#[derive(Default)]
pub struct ContextMap {
    gamepad: GamepadDevice,
    actions: Vec<ActionMap>,
    actions_data: ActionsData,
}

impl ContextMap {
    /// Associates context with gamepad.
    ///
    /// By default it's [`GamepadDevice::Any`].
    pub fn with_gamepad(gamepad: impl Into<GamepadDevice>) -> Self {
        Self {
            gamepad: gamepad.into(),
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
        entities: &[Entity],
        delta: f32,
    ) {
        for action_map in &mut self.actions {
            action_map.update(
                world,
                commands,
                reader,
                &mut self.actions_data,
                self.gamepad,
                entities,
                delta,
            );
        }
    }

    pub(super) fn trigger_removed(&self, commands: &mut Commands, entities: &[Entity]) {
        for action_map in &self.actions {
            let data = self
                .actions_data
                .get(&action_map.type_id)
                .expect("data and actions should have matching type IDs");
            data.trigger_removed(commands, entities, action_map.dim);
        }
    }
}

pub struct ActionMap {
    type_id: TypeId,
    action_name: &'static str,
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
            action_name: any::type_name::<A>(),
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
        self.with_axis2d([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD])
    }

    pub fn with_arrows(&mut self) -> &mut Self {
        self.with_axis2d([
            KeyCode::ArrowUp,
            KeyCode::ArrowLeft,
            KeyCode::ArrowDown,
            KeyCode::ArrowRight,
        ])
    }

    pub fn with_dpad(&mut self) -> &mut Self {
        self.with_axis2d([
            GamepadButtonType::DPadUp,
            GamepadButtonType::DPadLeft,
            GamepadButtonType::DPadDown,
            GamepadButtonType::DPadRight,
        ])
    }

    pub fn with_axis2d<I: Into<Input> + Copy>(&mut self, inputs: [I; 4]) -> &mut Self {
        self.with(InputMap::new(inputs[0].into()).with_modifier(SwizzleAxis::YXZ))
            .with(InputMap::new(inputs[1].into()).with_modifier(Negate))
            .with(
                InputMap::new(inputs[2].into())
                    .with_modifier(Negate)
                    .with_modifier(SwizzleAxis::YXZ),
            )
            .with(InputMap::new(inputs[3].into()))
    }

    pub fn with_stick(&mut self, stick: GamepadStick) -> &mut Self {
        self.with(stick.x())
            .with(InputMap::from(stick.y()).with_modifier(SwizzleAxis::YXZ))
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

    #[allow(clippy::too_many_arguments)]
    fn update(
        &mut self,
        world: &World,
        commands: &mut Commands,
        reader: &mut InputReader,
        actions_data: &mut ActionsData,
        gamepad: GamepadDevice,
        entities: &[Entity],
        delta: f32,
    ) {
        trace!("updating action `{}`", self.action_name);

        let mut tracker = TriggerTracker::new(ActionValue::zero(self.dim));
        for input_map in &mut self.inputs {
            if let Some(value) = reader.value(input_map.input, gamepad, self.consumes_input) {
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

        data.update(commands, entities, state, value, delta);
    }
}

pub struct InputMap {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,
}

impl InputMap {
    pub fn new(input: Input) -> Self {
        Self {
            input,
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
        Self::new(value.into())
    }
}

impl From<GamepadButtonType> for InputMap {
    fn from(value: GamepadButtonType) -> Self {
        Self::new(value.into())
    }
}

impl From<GamepadAxisType> for InputMap {
    fn from(value: GamepadAxisType) -> Self {
        Self::new(value.into())
    }
}

/// Represents the side of a gamepad's analog stick.
///
/// See also [`ActionMap::with_stick`].
#[derive(Clone, Copy, Debug)]
pub enum GamepadStick {
    /// Corresponds to [`GamepadAxisType::LeftStickX`] and [`GamepadAxisType::LeftStickY`]
    Left,
    /// Corresponds to [`GamepadAxisType::RightStickX`] and [`GamepadAxisType::RightStickY`]
    Right,
}

impl GamepadStick {
    /// Returns associated X axis.
    pub fn x(self) -> GamepadAxisType {
        match self {
            GamepadStick::Left => GamepadAxisType::LeftStickX,
            GamepadStick::Right => GamepadAxisType::RightStickX,
        }
    }

    /// Returns associated Y axis.
    pub fn y(self) -> GamepadAxisType {
        match self {
            GamepadStick::Left => GamepadAxisType::LeftStickY,
            GamepadStick::Right => GamepadAxisType::RightStickY,
        }
    }
}
