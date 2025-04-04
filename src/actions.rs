use alloc::vec::Vec;
use core::{
    any::{self, TypeId},
    fmt::Debug,
    marker::PhantomData,
};

use bevy::{
    prelude::*,
    utils::{Entry, HashMap},
};

use crate::{
    action_binding::ActionBinding,
    action_value::ActionValue,
    events::{ActionEvents, Canceled, Completed, Fired, Ongoing, Started},
    input::GamepadDevice,
    input_action::{ActionOutput, InputAction},
    input_reader::{InputReader, ResetInput},
};

/// Instance for [`ActionsMarker`].
///
/// Stores [`InputAction`]s and evaluates their [`ActionState`] in the order they are bound.
///
/// Each action can have multiple associated [`Input`]s, any of which can trigger the action.
///
/// Additionally, you can assign [`InputModifier`]s and [`InputCondition`]s at both the action
/// and input levels.
///
/// You can define bindings before the insertion, but it's recommended to create an observer
/// for [`Binding`](crate::action_instances::Binding). To setup bindings, register an observer
/// an obtain this component. This way you can conveniently reload bindings when you settings
/// change using [`RebuildBindings`](crate::action_instances::RebuildBindings).
///
/// Until the component is exists on the entity, actions will be evaluated and trigger [`events`](super::events).
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
/// [`InputCondition`]: crate::input_condition::InputCondition
/// [`InputCondition::kind`]: crate::input_condition::InputCondition::kind
/// [`InputModifier`]: crate::input_modifier::InputModifier
/// [`Input`]: crate::input::Input
#[derive(Component)]
pub struct Actions<M: ActionsMarker> {
    gamepad: GamepadDevice,
    bindings: Vec<ActionBinding>,
    actions: ActionsData,
    marker: PhantomData<M>,
}

impl<M: ActionsMarker> Actions<M> {
    /// Associates context with gamepad.
    ///
    /// By default it's [`GamepadDevice::Any`].
    pub fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        self.gamepad = gamepad.into();
    }

    /// Starts binding an action.
    ///
    /// This method can be called multiple times for the same action to extend its mappings.
    pub fn bind<A: InputAction>(&mut self) -> &mut ActionBinding {
        let type_id = TypeId::of::<A>();
        match self.actions.entry(type_id) {
            Entry::Occupied(_entry) => self
                .bindings
                .iter_mut()
                .find(|action_bind| action_bind.type_id() == type_id)
                .expect("actions and bindings should have matching type IDs"),
            Entry::Vacant(entry) => {
                entry.insert(ActionData::new::<A>());
                self.bindings.push(ActionBinding::new::<A>());
                self.bindings.last_mut().unwrap()
            }
        }
    }

    /// Returns associated bindings for action `A` if exists.
    ///
    /// For panicking version see [`Self::binding`].
    /// For assigning bindings use [`Self::bind`].
    pub fn get_binding<A: InputAction>(&self) -> Option<&ActionBinding> {
        self.bindings
            .iter()
            .find(|binding| binding.type_id() == TypeId::of::<A>())
    }

    /// Returns associated bindings for action `A`.
    ///
    /// For non-panicking version see [`Self::get_binding`].
    /// For assigning bindings use [`Self::bind`].
    pub fn binding<A: InputAction>(&self) -> &ActionBinding {
        self.get_binding::<A>().unwrap_or_else(|| {
            panic!(
                "action `{}` should be binded before access",
                any::type_name::<A>()
            )
        })
    }

    /// Returns associated state for action `A` if exists.
    ///
    /// For panicking version see [`Self::action`].
    pub fn get_action<A: InputAction>(&self) -> Option<&ActionData> {
        self.actions.action::<A>()
    }

    /// Returns associated state for action `A`.
    ///
    /// For non-panicking version see [`Self::get_action`].
    ///
    /// # Panics
    ///
    /// Panics if the action `A` was not bound beforehand.
    pub fn action<A: InputAction>(&self) -> &ActionData {
        self.get_action::<A>().unwrap_or_else(|| {
            panic!(
                "action `{}` should be binded before access",
                any::type_name::<A>()
            )
        })
    }

    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        reader.set_gamepad(self.gamepad);
        for binding in &mut self.bindings {
            binding.update(commands, reader, &mut self.actions, time, entity);
        }
    }

    /// Sets the state for each action to [`ActionState::None`]  and triggers transitions with zero value.
    ///
    /// Resets the input.
    pub(crate) fn reset(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        for binding in self.bindings.drain(..) {
            let action = self
                .actions
                .get_mut(&binding.type_id())
                .expect("actions and bindings should have matching type IDs");
            action.update(time, ActionState::None, ActionValue::zero(binding.dim()));
            action.trigger_events(commands, entity);
            if binding.require_reset() {
                reset_input.extend(binding.bindings().iter().map(|binding| binding.input));
            }
        }

        self.gamepad = Default::default();
        self.actions.clear();
    }
}

impl<M: ActionsMarker> Default for Actions<M> {
    fn default() -> Self {
        Self {
            gamepad: Default::default(),
            bindings: Default::default(),
            actions: Default::default(),
            marker: PhantomData,
        }
    }
}

/// Marker for [`Actions`].
///
/// # Examples
///
/// To implement the trait you can use the [`ActionsMarker`](bevy_enhanced_input_macros::ActionsMarker)
/// derive to reduce boilerplate:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(ActionsMarker)]
/// struct Player;
/// ```
///
/// Optionally you can pass `priority`:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(ActionsMarker)]
/// #[actions_marker(priority = 1)]
/// struct Player;
/// ```
pub trait ActionsMarker: Send + Sync + 'static {
    /// Determines the evaluation order of [`Actions<Self>`].
    ///
    /// Ordering is global.
    /// Contexts with a higher priority evaluated first.
    const PRIORITY: usize = 0;
}

/// Map for actions to their data.
///
/// Can be accessed from [`InputCondition::evaluate`](crate::input_condition::InputCondition::evaluate)
/// or [`Actions`].
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
    trigger_events: fn(&Self, &mut Commands, Entity),
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
    pub fn trigger_events(&self, commands: &mut Commands, entity: Entity) {
        (self.trigger_events)(self, commands, entity);
    }

    /// A typed version of [`Self::trigger_events`].
    fn trigger_events_typed<A: InputAction>(&self, commands: &mut Commands, entity: Entity) {
        for (_, event) in self.events.iter_names() {
            match event {
                ActionEvents::STARTED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Started::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                        },
                    );
                }
                ActionEvents::ONGOING => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Ongoing::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::FIRED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Fired::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                            fired_secs: self.fired_secs,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::CANCELED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Canceled::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::COMPLETED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
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

fn trigger_and_log<A, E: Event + Debug>(commands: &mut Commands, entity: Entity, event: E) {
    debug!(
        "triggering `{event:?}` for `{}` for `{entity}`",
        any::type_name::<A>()
    );
    commands.trigger_targets(event, entity);
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
    use bevy_enhanced_input_macros::{ActionsMarker, InputAction};

    use super::*;

    #[test]
    fn bind() {
        let mut actions = Actions::<Dummy>::default();
        actions.bind::<DummyAction>().to(KeyCode::KeyA);
        actions.bind::<DummyAction>().to(KeyCode::KeyB);
        assert_eq!(actions.bindings.len(), 1);

        let action = actions.binding::<DummyAction>();
        assert_eq!(action.bindings().len(), 2);
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;

    #[derive(ActionsMarker)]
    struct Dummy;
}
