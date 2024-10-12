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

/// Instance for [`InputContext`](super::InputContext).
///
/// Stores [`InputAction`]s and evaluates their [`ActionState`] in the order they are bound.
///
/// Each action can have multiple associated [`Input`]s, any of which can trigger the action.
///
/// Additionally, you can assign [`InputModifier`]s and [`InputCondition`]s at both the action
/// and input levels.
///
/// Action evaluation follows these steps:
///
/// 1. Iterate over each [`ActionValue`] from the associated [`Input`]s:
///    1.1. Apply input-level [`InputModifier`]s.
///    1.2. Evaluate input-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
/// 2. Select all [`ActionValue`]s with the most significant [`ActionState`] and combine based on [`InputAction::ACCUMULATION`].
/// 3. Apply action level [`InputModifier`]s.
/// 4. Evaluate action level [`InputCondition`]s, combining their results according to [`InputCondition::kind`].
/// 5. Set the final [`ActionState`] based on the results.
///
/// [`ActionState`]: super::input_action::ActionState
#[derive(Default)]
pub struct ContextInstance {
    gamepad: GamepadDevice,
    actions: Vec<ActionMap>,
    actions_data: ActionsData,
}

impl ContextInstance {
    /// Associates context with gamepad.
    ///
    /// By default it's [`GamepadDevice::Any`].
    pub fn with_gamepad(gamepad: impl Into<GamepadDevice>) -> Self {
        Self {
            gamepad: gamepad.into(),
            ..Default::default()
        }
    }

    /// Starts binding an action.
    ///
    /// This method can be called multiple times for the same action to extend its mappings.
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

/// [`InputAction`]'s bindings for [`ContextInstance`].
pub struct ActionMap {
    type_id: TypeId,
    action_name: &'static str,
    consumes_input: bool,
    accumulation: Accumulation,
    dim: ActionValueDim,

    modifiers: Vec<Box<dyn InputModifier>>,
    conditions: Vec<Box<dyn InputCondition>>,
    inputs: Vec<(ActionValue, InputMap)>,
}

impl ActionMap {
    #[must_use]
    fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            action_name: any::type_name::<A>(),
            dim: A::DIM,
            consumes_input: A::CONSUMES_INPUT,
            accumulation: A::ACCUMULATION,
            modifiers: Default::default(),
            conditions: Default::default(),
            inputs: Default::default(),
        }
    }

    /// Maps WASD keys as 2-dimentional input.
    ///
    /// See also [`Self::with_axis2d`].
    pub fn with_wasd(&mut self) -> &mut Self {
        self.with_axis2d(KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD)
    }

    /// Maps keyboard arrow keys as 2-dimentional input.
    ///
    /// See also [`Self::with_axis2d`].
    pub fn with_arrows(&mut self) -> &mut Self {
        self.with_axis2d(
            KeyCode::ArrowUp,
            KeyCode::ArrowLeft,
            KeyCode::ArrowDown,
            KeyCode::ArrowRight,
        )
    }

    /// Maps D-pad as 2-dimentional input.
    ///
    /// See also [`Self::with_axis2d`].
    pub fn with_dpad(&mut self) -> &mut Self {
        self.with_axis2d(
            GamepadButtonType::DPadUp,
            GamepadButtonType::DPadLeft,
            GamepadButtonType::DPadDown,
            GamepadButtonType::DPadRight,
        )
    }

    /// Maps 4 keys as 2-dimentional input.
    ///
    /// This is a convenience "preset" that uses [`SwizzleAxis`] and [`Negate`] to
    /// bind the keys to cardinal directions.
    pub fn with_axis2d<I: Into<Input>>(&mut self, up: I, left: I, down: I, right: I) -> &mut Self {
        self.with(InputMap::from(up.into()).with_modifier(SwizzleAxis::YXZ))
            .with(InputMap::from(left.into()).with_modifier(Negate::default()))
            .with(
                InputMap::from(down.into())
                    .with_modifier(Negate::default())
                    .with_modifier(SwizzleAxis::YXZ),
            )
            .with(InputMap::from(right.into()))
    }

    /// Maps the given stick as 2-dimentional input.
    pub fn with_stick(&mut self, stick: GamepadStick) -> &mut Self {
        self.with(stick.x())
            .with(InputMap::from(stick.y()).with_modifier(SwizzleAxis::YXZ))
    }

    /// Adds action-level modifier.
    pub fn with_modifier(&mut self, modifier: impl InputModifier) -> &mut Self {
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Adds action-level condition.
    pub fn with_condition(&mut self, condition: impl InputCondition) -> &mut Self {
        self.conditions.push(Box::new(condition));
        self
    }

    /// Adds input mapping.
    ///
    /// The action can be triggered by any input mapping. If multiple input mappings
    /// return [`ActionState`](super::input_action::ActionState).
    ///
    /// Thanks to [`Into`] impls, it can be called directly with buttons/axes:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .with(KeyCode::Space)
    ///     .with(GamepadButtonType::South);
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(dim = Bool)]
    /// # struct Jump;
    /// ```
    ///
    /// or with [`Input`] if you want to assign keyboard modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>().with(Input::Keyboard {
    ///     key_code: KeyCode::Space,
    ///     modifiers: KeyboardModifiers::CONTROL,
    /// });
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(dim = Bool)]
    /// # struct Jump;
    /// ```
    ///
    /// If you want input with modifiers or conditions,
    /// you will need to wrap it into [`InputMap`]:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .with(InputMap::from(KeyCode::Space).with_condition(Down::default()));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(dim = Bool)]
    /// # struct Jump;
    /// ```
    pub fn with(&mut self, map: impl Into<InputMap>) -> &mut Self {
        self.inputs.push((ActionValue::zero(self.dim), map.into()));
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
        for (value, input_map) in &mut self.inputs {
            if let Some(new_value) = reader.value(input_map.input, gamepad, self.consumes_input) {
                // Retain the old value and update it if a new one
                // is available since the reader is event-based.
                *value = new_value.convert(self.dim);
            }
            let mut current_tracker = TriggerTracker::new(*value);
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

/// Associated input for [`ActionMap`].
pub struct InputMap {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,
}

impl InputMap {
    /// Adds modifier.
    #[must_use]
    pub fn with_modifier(mut self, modifier: impl InputModifier) -> Self {
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Adds condition.
    #[must_use]
    pub fn with_condition(mut self, condition: impl InputCondition) -> Self {
        self.conditions.push(Box::new(condition));
        self
    }
}

impl From<KeyCode> for InputMap {
    fn from(value: KeyCode) -> Self {
        Self::from(Input::from(value))
    }
}

impl From<GamepadButtonType> for InputMap {
    fn from(value: GamepadButtonType) -> Self {
        Self::from(Input::from(value))
    }
}

impl From<GamepadAxisType> for InputMap {
    fn from(value: GamepadAxisType) -> Self {
        Self::from(Input::from(value))
    }
}

impl From<Input> for InputMap {
    fn from(input: Input) -> Self {
        Self {
            input,
            modifiers: Default::default(),
            conditions: Default::default(),
        }
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
