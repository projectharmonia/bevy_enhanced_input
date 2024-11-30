mod trigger_tracker;

use std::{
    any::{self, TypeId},
    cmp::Ordering,
    fmt::Debug,
};

use bevy::{
    prelude::*,
    utils::{Entry, HashMap},
};

use super::{
    events::{ActionEvents, Canceled, Completed, Fired, Ongoing, Started},
    input_action::{Accumulation, ActionOutput, InputAction},
    input_condition::InputCondition,
    input_modifier::{negate::Negate, swizzle_axis::SwizzleAxis, InputModifier},
};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input::{input_reader::InputReader, GamepadDevice, Input},
};
use trigger_tracker::TriggerTracker;

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
///    Combined value be converted into [`ActionOutput::DIM`] using [`ActionValue::convert`].
/// 3. Apply action level [`InputModifier`]s.
/// 4. Evaluate action level [`InputCondition`]s, combining their results according to [`InputCondition::kind`].
/// 5. Set the final [`ActionState`] based on the results.
///    Final value be converted into [`InputAction::Output`] using [`ActionOutput::convert_from`].
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

    /// Returns associated state for action `A`.
    ///
    /// See also [`ContextInstances::get`](super::ContextInstances::get).
    pub fn action<A: InputAction>(&self) -> Option<&ActionData> {
        self.actions.action::<A>()
    }

    pub(super) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        entities: &[Entity],
    ) {
        reader.set_gamepad(self.gamepad);
        for binding in &mut self.bindings {
            binding.update(commands, reader, &mut self.actions, time, entities);
        }
    }

    /// Copies [`ActionData`] for each binding and triggers transition to [`ActionState::None`] with zero value.
    ///
    /// Instance data remains unchanges.
    pub(super) fn trigger_removed(
        &self,
        commands: &mut Commands,
        time: &Time<Virtual>,
        entities: &[Entity],
    ) {
        for binding in &self.bindings {
            let mut action = *self
                .actions
                .get(&binding.type_id)
                .expect("actions and bindings should have matching type IDs");
            action.update(time, ActionState::None, ActionValue::zero(binding.dim));
            action.trigger_events(commands, entities);
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
    bindings: Vec<InputBind>,

    /// Consumed inputs during state evaluation.
    consume_buffer: Vec<Input>,
}

impl ActionBind {
    #[must_use]
    fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            action_name: any::type_name::<A>(),
            dim: A::Output::DIM,
            consume_input: A::CONSUME_INPUT,
            accumulation: A::ACCUMULATION,
            modifiers: Default::default(),
            conditions: Default::default(),
            bindings: Default::default(),
            consume_buffer: Default::default(),
        }
    }

    /// Maps WASD keys as 2-dimentional input.
    ///
    /// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
    /// toward the camera. To map movement correctly in 3D space, you will
    /// need to invert Y and apply it to Z translation inside your observer.
    ///
    /// Shorthand for [`Self::with_xy_axis`].
    pub fn with_wasd(&mut self) -> &mut Self {
        self.with_xy_axis(KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD)
    }

    /// Maps keyboard arrow keys as 2-dimentional input.
    ///
    /// Shorthand for [`Self::with_xy_axis`].
    /// See also [`Self::with_wasd`].
    pub fn with_arrows(&mut self) -> &mut Self {
        self.with_xy_axis(
            KeyCode::ArrowUp,
            KeyCode::ArrowLeft,
            KeyCode::ArrowDown,
            KeyCode::ArrowRight,
        )
    }

    /// Maps D-pad as 2-dimentional input.
    ///
    /// Shorthand for [`Self::with_xy_axis`].
    /// See also [`Self::with_wasd`].
    pub fn with_dpad(&mut self) -> &mut Self {
        self.with_xy_axis(
            GamepadButtonType::DPadUp,
            GamepadButtonType::DPadLeft,
            GamepadButtonType::DPadDown,
            GamepadButtonType::DPadRight,
        )
    }

    /// Maps 4 buttons as 2-dimentional input.
    ///
    /// This is a convenience "preset" that uses [`SwizzleAxis`] and [`Negate`] to
    /// bind the buttons to X and Y axes.
    ///
    /// The order of arguments follows the common "WASD" mapping.
    pub fn with_xy_axis<I: Into<Input>>(&mut self, up: I, left: I, down: I, right: I) -> &mut Self {
        self.to(up.with_modifier(SwizzleAxis::YXZ))
            .to(left.with_modifier(Negate::default()))
            .to(down
                .with_modifier(Negate::default())
                .with_modifier(SwizzleAxis::YXZ))
            .to(right)
    }

    /// Maps the given stick as 2-dimentional input.
    pub fn with_stick(&mut self, stick: GamepadStick) -> &mut Self {
        self.to(stick.x())
            .to(stick.y())
            .with_modifier(SwizzleAxis::YXZ)
    }

    /// Adds action-level modifier.
    pub fn with_modifier(&mut self, modifier: impl InputModifier) -> &mut Self {
        debug!("adding `{modifier:?}` to `{}`", self.action_name);
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Adds action-level condition.
    pub fn with_condition(&mut self, condition: impl InputCondition) -> &mut Self {
        debug!("adding `{condition:?}` to `{}`", self.action_name);
        self.conditions.push(Box::new(condition));
        self
    }

    /// Adds input mapping.
    ///
    /// The action can be triggered by any input mapping. If multiple input mappings
    /// return [`ActionState`].
    ///
    /// Thanks to [`Into`] impls, it can be called directly with buttons/axes:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .to(GamepadButtonType::South);
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// or with [`Input`] if you want to assign keyboard modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>().to(Input::Keyboard {
    ///     key: KeyCode::Space,
    ///     mod_keys: ModKeys::CONTROL,
    /// });
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// If you want input with modifiers or conditions,
    /// you will need to wrap it into [`InputBind`]:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(InputBind::new(KeyCode::Space).with_condition(Release::default()));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    pub fn to(&mut self, binding: impl Into<InputBind>) -> &mut Self {
        let binding = binding.into();
        debug!("adding `{binding:?}` to `{}`", self.action_name);
        self.bindings.push(binding);
        self
    }

    fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        actions: &mut ActionsData,
        time: &Time<Virtual>,
        entities: &[Entity],
    ) {
        trace!("updating action `{}`", self.action_name);

        let mut tracker = TriggerTracker::new(ActionValue::zero(self.dim));
        for binding in &mut self.bindings {
            let value = reader.value(binding.input);
            if binding.ignored {
                // Ignore until we read zero for this mapping.
                if value.as_bool() {
                    continue;
                } else {
                    binding.ignored = false;
                }
            }

            let mut current_tracker = TriggerTracker::new(value);
            current_tracker.apply_modifiers(actions, time, &mut binding.modifiers);
            current_tracker.apply_conditions(actions, time, &mut binding.conditions);

            let current_state = current_tracker.state();
            if current_state == ActionState::None {
                // Ignore non-active trackers to allow the action to fire even if all
                // input-level conditions return `ActionState::None`. This ensures that an
                // action-level condition or modifier can still trigger the action.
                continue;
            }

            match current_state.cmp(&tracker.state()) {
                Ordering::Less => (),
                Ordering::Equal => {
                    tracker.combine(current_tracker, self.accumulation);
                    if self.consume_input {
                        self.consume_buffer.push(binding.input);
                    }
                }
                Ordering::Greater => {
                    tracker.overwrite(current_tracker);
                    if self.consume_input {
                        self.consume_buffer.clear();
                        self.consume_buffer.push(binding.input);
                    }
                }
            }
        }

        tracker.apply_modifiers(actions, time, &mut self.modifiers);
        tracker.apply_conditions(actions, time, &mut self.conditions);

        let action = actions
            .get_mut(&self.type_id)
            .expect("actions and bindings should have matching type IDs");

        let state = tracker.state();
        let value = tracker.value().convert(self.dim);

        if self.consume_input {
            if state != ActionState::None {
                for &input in &self.consume_buffer {
                    reader.consume(input);
                }
            }
            self.consume_buffer.clear();
        }

        action.update(time, state, value);
        if !tracker.events_blocked() {
            action.trigger_events(commands, entities);
        }
    }
}

/// Map for actions to their data.
///
/// Can be accessed from [`InputCondition::evaluate`]
/// or [`ContextInstances::get`](super::ContextInstances::get).
#[derive(Default, Deref, DerefMut)]
pub struct ActionsData(pub HashMap<TypeId, ActionData>);

impl ActionsData {
    /// Returns associated state for action `A`.
    pub fn action<A: InputAction>(&self) -> Option<&ActionData> {
        self.get(&TypeId::of::<A>())
    }

    /// Inserts a state for action `A`.
    ///
    /// Returns previosly associated state if present.
    pub fn insert_action<A: InputAction>(&mut self, action: ActionData) -> Option<ActionData> {
        self.insert(TypeId::of::<A>(), action)
    }
}

/// Tracker for action state.
///
/// Stored inside [`ActionsData`].
#[derive(Clone, Copy)]
pub struct ActionData {
    state: ActionState,
    events: ActionEvents,
    value: ActionValue,
    elapsed_secs: f32,
    fired_secs: f32,
    trigger_events: fn(&Self, &mut Commands, &[Entity]),
}

impl ActionData {
    /// Creates a new instance associated with action `A`.
    ///
    /// [`Self::trigger_events`] will trigger events for `A`.
    #[must_use]
    pub fn new<A: InputAction>() -> Self {
        Self {
            state: Default::default(),
            events: ActionEvents::empty(),
            value: ActionValue::zero(A::Output::DIM),
            elapsed_secs: 0.0,
            fired_secs: 0.0,
            trigger_events: Self::trigger_events_typed::<A>,
        }
    }

    /// Updates internal state.
    pub fn update(
        &mut self,
        time: &Time<Virtual>,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) {
        match self.state {
            ActionState::None => {
                self.elapsed_secs = 0.0;
                self.fired_secs = 0.0;
            }
            ActionState::Ongoing => {
                self.elapsed_secs += time.delta_seconds();
                self.fired_secs = 0.0;
            }
            ActionState::Fired => {
                self.elapsed_secs += time.delta_seconds();
                self.fired_secs += time.delta_seconds();
            }
        }

        self.events = ActionEvents::new(self.state, state);
        self.state = state;
        self.value = value.into();
    }

    /// Triggers events resulting from a state transition after [`Self::update`].
    ///
    /// See also [`Self::new`].
    pub fn trigger_events(&self, commands: &mut Commands, entities: &[Entity]) {
        (self.trigger_events)(self, commands, entities);
    }

    /// A typed version of [`Self::trigger_events`].
    fn trigger_events_typed<A: InputAction>(&self, commands: &mut Commands, entities: &[Entity]) {
        for (_, event) in self.events.iter_names() {
            match event {
                ActionEvents::STARTED => {
                    trigger_for_each(
                        commands,
                        entities,
                        Started::<A> {
                            value: A::Output::convert_from(self.value),
                            state: self.state,
                        },
                    );
                }
                ActionEvents::ONGOING => {
                    trigger_for_each(
                        commands,
                        entities,
                        Ongoing::<A> {
                            value: A::Output::convert_from(self.value),
                            state: self.state,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::FIRED => {
                    trigger_for_each(
                        commands,
                        entities,
                        Fired::<A> {
                            value: A::Output::convert_from(self.value),
                            state: self.state,
                            fired_secs: self.fired_secs,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::CANCELED => {
                    trigger_for_each(
                        commands,
                        entities,
                        Canceled::<A> {
                            value: A::Output::convert_from(self.value),
                            state: self.state,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::COMPLETED => {
                    trigger_for_each(
                        commands,
                        entities,
                        Completed::<A> {
                            value: A::Output::convert_from(self.value),
                            state: self.state,
                            fired_secs: self.fired_secs,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                _ => unreachable!("iteration should yield only named flags"),
            }
        }
    }

    /// Returns the current state.
    pub fn state(&self) -> ActionState {
        self.state
    }

    /// Returns events triggered by a transition of [`Self::state`] since the last update.
    pub fn events(&self) -> ActionEvents {
        self.events
    }

    /// Returns the value since the last update.
    pub fn value(&self) -> ActionValue {
        self.value
    }

    /// Time the action was in [`ActionState::Ongoing`] and [`ActionState::Fired`] states.
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed_secs
    }

    /// Time the action was in [`ActionState::Fired`] state.
    pub fn fired_secs(&self) -> f32 {
        self.fired_secs
    }
}

/// Triggers a copyable event for each entity separately and logs it.
///
// It's cheaper to copy the event than to clone the entities.
fn trigger_for_each<E: Event + Debug + Clone + Copy>(
    commands: &mut Commands,
    entities: &[Entity],
    event: E,
) {
    for &entity in entities {
        trace!("triggering `{event:?}` for `{entity}`");
        commands.trigger_targets(event, entity);
    }
}

/// State for [`ActionData`].
///
/// States are ordered by their significance.
///
/// See also [`ActionEvents`].
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionState {
    /// Condition is not triggered.
    #[default]
    None,
    /// Condition has started triggering, but has not yet finished.
    ///
    /// For example, [`Hold`](super::input_condition::hold::Hold) condition
    /// requires its state to be maintained over several frames.
    Ongoing,
    /// The condition has been met.
    Fired,
}

/// Associated input for [`ActionBind`].
#[derive(Debug)]
pub struct InputBind {
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

impl InputBind {
    /// Creates a new instance without modifiers and conditions.
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            modifiers: Default::default(),
            conditions: Default::default(),
            ignored: true,
        }
    }
}

impl<I: Into<Input>> From<I> for InputBind {
    fn from(input: I) -> Self {
        Self::new(input)
    }
}

/// A trait to ergonomically add a modifier or condition to any type that can be converted into a binding.
pub trait InputBindModCond {
    /// Adds modifier.
    #[must_use]
    fn with_modifier(self, modifier: impl InputModifier) -> InputBind;

    /// Adds condition.
    #[must_use]
    fn with_condition(self, condition: impl InputCondition) -> InputBind;
}

impl<T: Into<InputBind>> InputBindModCond for T {
    fn with_modifier(self, modifier: impl InputModifier) -> InputBind {
        let mut binding = self.into();
        binding.modifiers.push(Box::new(modifier));
        binding
    }

    fn with_condition(self, condition: impl InputCondition) -> InputBind {
        let mut binding = self.into();
        binding.conditions.push(Box::new(condition));
        binding
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

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;

    #[test]
    fn bind() {
        let mut ctx = ContextInstance::default();
        ctx.bind::<DummyAction>().to(KeyCode::KeyA);
        ctx.bind::<DummyAction>().to(KeyCode::KeyB);
        assert_eq!(ctx.bindings.len(), 1);

        let action = ctx.bindings.first().unwrap();
        assert_eq!(action.bindings.len(), 2);
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;
}
