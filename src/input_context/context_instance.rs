use std::any::{self, TypeId};

use bevy::{prelude::*, utils::Entry};

use super::{
    input_action::{Accumulation, ActionData, ActionsData, InputAction},
    input_condition::InputCondition,
    input_modifier::{negate::Negate, swizzle_axis::SwizzleAxis, InputModifier},
    trigger_tracker::TriggerTracker,
};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input::{input_reader::InputReader, GamepadDevice, Input},
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
/// New instances won't react to currently held inputs until they are released.
/// This prevents unintended behavior where switching contexts using the same key
/// could cause an immediate switch back, as buttons are rarely pressed for only a single frame.
///
/// [`ActionState`]: super::input_action::ActionState
#[derive(Default)]
pub struct ContextInstance {
    gamepad: GamepadDevice,
    bindings: Vec<ActionBind>,
    actions: ActionsData,
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
    pub fn bind<A: InputAction>(&mut self) -> &mut ActionBind {
        let type_id = TypeId::of::<A>();
        match self.actions.entry(type_id) {
            Entry::Occupied(_entry) => self
                .bindings
                .iter_mut()
                .find(|binding| binding.type_id == type_id)
                .expect("actions and bindings should have matching type IDs"),
            Entry::Vacant(entry) => {
                entry.insert(ActionData::new::<A>());
                self.bindings.push(ActionBind::new::<A>());
                self.bindings.last_mut().unwrap()
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
        reader.set_gamepad(self.gamepad);
        for binding in &mut self.bindings {
            binding.update(world, commands, reader, &mut self.actions, entities, delta);
        }
    }

    pub(super) fn trigger_removed(&self, commands: &mut Commands, entities: &[Entity]) {
        for binding in &self.bindings {
            let action = self
                .actions
                .get(&binding.type_id)
                .expect("actions and bindings should have matching type IDs");
            action.trigger_removed(commands, entities, binding.dim);
        }
    }
}

/// Bindings of [`InputAction`] for [`ContextInstance`].
///
/// These bindings are stored separately from [`ActionsData`] to allow a currently
/// evaluating action to access the state of other actions.
pub struct ActionBind {
    type_id: TypeId,
    action_name: &'static str,
    consume_input: bool,
    accumulation: Accumulation,
    dim: ActionValueDim,

    modifiers: Vec<Box<dyn InputModifier>>,
    conditions: Vec<Box<dyn InputCondition>>,
    inputs: Vec<InputMap>,
}

impl ActionBind {
    #[must_use]
    fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            action_name: any::type_name::<A>(),
            dim: A::DIM,
            consume_input: A::CONSUME_INPUT,
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
        self.with(InputMap::new(up).with_modifier(SwizzleAxis::YXZ))
            .with(InputMap::new(left).with_modifier(Negate::default()))
            .with(
                InputMap::new(down)
                    .with_modifier(Negate::default())
                    .with_modifier(SwizzleAxis::YXZ),
            )
            .with(InputMap::new(right))
    }

    /// Maps the given stick as 2-dimentional input.
    pub fn with_stick(&mut self, stick: GamepadStick) -> &mut Self {
        self.with(stick.x())
            .with(InputMap::new(stick.y()).with_modifier(SwizzleAxis::YXZ))
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
    ///     key: KeyCode::Space,
    ///     modifiers: Modifiers::CONTROL,
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
    ///     .with(InputMap::new(KeyCode::Space).with_condition(Down::default()));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(dim = Bool)]
    /// # struct Jump;
    /// ```
    pub fn with(&mut self, map: impl Into<InputMap>) -> &mut Self {
        self.inputs.push(map.into());
        self
    }

    fn update(
        &mut self,
        world: &World,
        commands: &mut Commands,
        reader: &mut InputReader,
        actions: &mut ActionsData,
        entities: &[Entity],
        delta: f32,
    ) {
        trace!("updating action `{}`", self.action_name);

        reader.set_consume_input(self.consume_input);
        let mut tracker = TriggerTracker::new(ActionValue::zero(self.dim));
        for input_map in &mut self.inputs {
            let value = reader.value(input_map.input).convert(self.dim);
            if input_map.ignored {
                // Ignore until we read zero for this mapping.
                if value.as_bool() {
                    continue;
                } else {
                    input_map.ignored = false;
                }
            }

            let mut current_tracker = TriggerTracker::new(value);
            current_tracker.apply_modifiers(world, delta, &mut input_map.modifiers);
            current_tracker.apply_conditions(world, actions, delta, &mut input_map.conditions);
            tracker.merge(current_tracker, self.accumulation);
        }

        tracker.apply_modifiers(world, delta, &mut self.modifiers);
        tracker.apply_conditions(world, actions, delta, &mut self.conditions);

        let (state, value) = tracker.finish();
        let action = actions
            .get_mut(&self.type_id)
            .expect("actions and bindings should have matching type IDs");

        action.update(commands, entities, state, value, delta);
    }
}

/// Associated input for [`ActionBind`].
pub struct InputMap {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,

    /// Newly created mappings are ignored by default until until a zero
    /// value is read for them.
    ///
    /// This prevents newly created contexts from reacting to currently
    /// held inputs until they are released.
    ignored: bool,
}

impl InputMap {
    /// Creates a new instance without modifiers and conditions.
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            modifiers: Default::default(),
            conditions: Default::default(),
            ignored: true,
        }
    }

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
        Self::new(value)
    }
}

impl From<GamepadButtonType> for InputMap {
    fn from(value: GamepadButtonType) -> Self {
        Self::new(value)
    }
}

impl From<GamepadAxisType> for InputMap {
    fn from(value: GamepadAxisType) -> Self {
        Self::new(value)
    }
}

impl From<Input> for InputMap {
    fn from(input: Input) -> Self {
        Self::new(input)
    }
}

/// Represents the side of a gamepad's analog stick.
///
/// See also [`ActionBind::with_stick`].
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
