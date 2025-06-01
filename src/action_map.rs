use core::{
    any::{self, TypeId},
    fmt::Debug,
};

use bevy::{platform::collections::HashMap, prelude::*};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{input_action::ActionOutput, prelude::*};

/// Maps markers that implement [`InputAction`] to their data (state, value, etc.).
///
/// Stored inside [`Actions`].
///
/// Accessible from [`InputCondition::evaluate`] and [`InputModifier::apply`].
#[derive(Default, Deref, DerefMut)]
pub struct ActionMap(pub HashMap<TypeId, Action>);

impl ActionMap {
    /// Returns associated state for action `A`.
    pub fn action<A: InputAction>(&self) -> Option<&Action> {
        self.get(&TypeId::of::<A>())
    }

    /// Inserts a state for action `A`.
    ///
    /// Returns previously associated state if present.
    pub fn insert_action<A: InputAction>(&mut self, action: Action) -> Option<Action> {
        self.insert(TypeId::of::<A>(), action)
    }
}

/// Data associated with an [`InputAction`] marker.
///
/// Stored inside [`ActionMap`].
///
/// This struct could also be created manually to track state for an action
/// with externally sourced data (e.g., network). Use [`Self::update`] to apply
/// the data followed by [`Self::trigger_events`].
#[derive(Clone, Copy)]
pub struct Action {
    state: ActionState,
    events: ActionEvents,
    value: ActionValue,
    elapsed_secs: f32,
    fired_secs: f32,
    trigger_events: fn(&Self, &mut Commands, Entity),
}

impl Action {
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
    /// See also [`Self::new`] and [`ActionEvents`].
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
    ///
    /// Unlike when reading values from triggers, this returns [`ActionValue`] since actions
    /// are stored in a type-erased format.
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

/// State for [`Action`].
///
/// States are ordered by their significance.
///
/// See also [`ActionEvents`] and [`ActionBinding`]().
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionState {
    /// Condition is not triggered.
    #[default]
    None,
    /// Condition has started triggering, but has not yet finished.
    ///
    /// For example, [`Hold`] condition requires its state to be
    /// maintained over several frames.
    Ongoing,
    /// The condition has been met.
    Fired,
}
