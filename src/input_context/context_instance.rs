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
    input_bind::{InputBind, InputBindings},
    input_condition::{InputCondition, InputConditions},
    input_modifier::{InputModifier, InputModifiers},
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
///    Final value be converted into [`InputAction::Output`] using [`ActionValue::convert`].
///
/// New instances won't react to currently held inputs until they are released.
/// This prevents unintended behavior where switching contexts using the same key
/// could cause an immediate switch back, as buttons are rarely pressed for only a single frame.
///
/// [`ActionState`]: super::context_instance::ActionState
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
    pub fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        self.gamepad = gamepad.into();
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

    /// Adds action-level modifiers.
    ///
    /// # Examples
    ///
    /// Single modifier:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_modifiers(Scale::splat(2.0)));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Jump;
    /// ```
    ///
    /// Multiple modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_modifiers((Scale::splat(2.0), Negate::all())));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Jump;
    /// ```
    pub fn with_modifiers(&mut self, modifiers: impl InputModifiers) -> &mut Self {
        for modifier in modifiers.iter_modifiers() {
            debug!("adding `{modifier:?}` to `{}`", self.action_name);
            self.modifiers.push(modifier);
        }

        self
    }

    /// Adds action-level conditions.
    ///
    /// # Examples
    ///
    /// Single condition:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_conditions(Release::default()));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// Multiple conditions:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_conditions((Release::default(), JustPress::default())));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    pub fn with_conditions(&mut self, conditions: impl InputConditions) -> &mut Self {
        for condition in conditions.iter_conditions() {
            debug!("adding `{condition:?}` to `{}`", self.action_name);
            self.conditions.push(condition);
        }

        self
    }

    /// Adds input mapping.
    ///
    /// The action can be triggered by any input mapping. If multiple input mappings
    /// return [`ActionState`], the behavior is determined by [`InputAction::ACCUMULATION`].
    ///
    /// Thanks to traits, this function can be called with multiple types:
    ///
    /// 1. Raw input types.
    /// 2. [`Input`] enum which wraps any supported raw input and can store keyboard modifiers.
    /// 3. [`InputBind`] which wraps [`Input`] and can store input modifiers or conditions.
    /// 4. [`InputBindings`] which wraps [`InputBind`] and can store multiple [`InputBind`]s.
    ///    Also implemented on tuples, so you can pass multiple inputs to a single call.
    ///
    /// # Examples
    ///
    /// Raw input:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to((KeyCode::Space, GamepadButton::South));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// Raw input with keyboard modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>().to(KeyCode::Space.with_mod_keys(ModKeys::CONTROL));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// Raw input with input conditions or modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>().to(KeyCode::Space.with_conditions(Release::default()));
    /// ctx.bind::<Attack>().to(MouseButton::Left.with_modifiers(Scale::splat(10.0)));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Attack;
    /// ```
    ///
    /// [`Input`] type directly:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Zoom>().to(Input::mouse_wheel());
    /// ctx.bind::<Move>().to(Input::mouse_motion());
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Zoom;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = Vec2)]
    /// # struct Move;
    /// ```
    ///
    /// Convenience preset which consists of multiple inputs
    /// with predefined conditions and modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Move>().to(Cardinal::wasd_keys());
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = Vec2)]
    /// # struct Move;
    /// ```
    ///
    /// Multiple buttons from settings:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// # let mut settings = KeyboardSettings::default();
    /// ctx.bind::<Inspect>().to(&settings.inspect);
    ///
    /// # #[derive(Default)]
    /// struct KeyboardSettings {
    ///     inspect: Vec<KeyCode>,
    /// }
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = Vec2)]
    /// # struct Inspect;
    /// ```
    pub fn to(&mut self, bindings: impl InputBindings) -> &mut Self {
        for binding in bindings.iter_bindings() {
            debug!("adding `{binding:?}` to `{}`", self.action_name);
            self.bindings.push(binding);
        }
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
    /// Returns previously associated state if present.
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
                self.elapsed_secs += time.delta_secs();
                self.fired_secs = 0.0;
            }
            ActionState::Fired => {
                self.elapsed_secs += time.delta_secs();
                self.fired_secs += time.delta_secs();
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
                            value: A::Output::as_output(self.value),
                            state: self.state,
                        },
                    );
                }
                ActionEvents::ONGOING => {
                    trigger_for_each(
                        commands,
                        entities,
                        Ongoing::<A> {
                            value: A::Output::as_output(self.value),
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
                            value: A::Output::as_output(self.value),
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
                            value: A::Output::as_output(self.value),
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
                            value: A::Output::as_output(self.value),
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
