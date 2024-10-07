use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use bevy::{prelude::*, utils::HashMap};

use crate::action_value::{ActionValue, ActionValueDim};

#[derive(Deref, Default, DerefMut)]
pub struct ActionsData(HashMap<TypeId, ActionData>);

impl ActionsData {
    pub fn get_action<A: InputAction>(&self) -> Option<&ActionData> {
        self.get(&TypeId::of::<A>())
    }
}

impl From<HashMap<TypeId, ActionData>> for ActionsData {
    fn from(value: HashMap<TypeId, ActionData>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy)]
pub struct ActionData {
    state: ActionState,
    elapsed_secs: f32,
    fired_secs: f32,
    trigger_events: fn(&Self, &mut Commands, &[Entity], ActionState, ActionValue),
}

impl ActionData {
    pub fn new<A: InputAction>() -> Self {
        Self {
            state: Default::default(),
            elapsed_secs: 0.0,
            fired_secs: 0.0,
            trigger_events: Self::trigger::<A>,
        }
    }

    pub fn update(
        &mut self,
        commands: &mut Commands,
        entities: &[Entity],
        state: ActionState,
        value: ActionValue,
        delta: f32,
    ) {
        // Add time from the previous frame if needed
        // before triggering events.
        match self.state {
            ActionState::None => (),
            ActionState::Ongoing => {
                self.elapsed_secs += delta;
            }
            ActionState::Fired => {
                self.elapsed_secs += delta;
                self.fired_secs += delta;
            }
        }

        (self.trigger_events)(self, commands, entities, state, value);

        // Reset time for updated state.
        self.state = state;
        match self.state {
            ActionState::None => {
                self.elapsed_secs = 0.0;
                self.fired_secs = 0.0;
            }
            ActionState::Ongoing => {
                self.fired_secs = 0.0;
            }
            ActionState::Fired => (),
        }
    }

    pub fn trigger_removed(
        &self,
        commands: &mut Commands,
        entities: &[Entity],
        dim: ActionValueDim,
    ) {
        (self.trigger_events)(
            self,
            commands,
            entities,
            ActionState::None,
            ActionValue::zero(dim),
        );
    }

    fn trigger<A: InputAction>(
        &self,
        commands: &mut Commands,
        entities: &[Entity],
        state: ActionState,
        value: ActionValue,
    ) {
        trace!(
            "changing state from `{:?}` to `{state:?}` with `{value:?}`",
            self.state
        );

        // Trigger an event for each entity separately
        // since it's cheaper to copy the event than to clone the entities.
        for &entity in entities {
            match (self.state(), state) {
                (ActionState::None, ActionState::None) => (),
                (ActionState::None, ActionState::Ongoing) => commands.trigger_targets(
                    ActionEvent::<A>::from(ActionEventKind::Started { value }),
                    entity,
                ),
                (ActionState::None, ActionState::Fired) => {
                    commands.trigger_targets(
                        ActionEvent::<A>::from(ActionEventKind::Started { value }),
                        entity,
                    );
                    commands.trigger_targets(
                        ActionEvent::<A>::from(ActionEventKind::Fired {
                            value,
                            fired_secs: 0.0,
                            elapsed_secs: 0.0,
                        }),
                        entity,
                    );
                }
                (ActionState::Ongoing, ActionState::None) => commands.trigger_targets(
                    ActionEvent::<A>::from(ActionEventKind::Canceled {
                        value,
                        elapsed_secs: self.elapsed_secs,
                    }),
                    entity,
                ),
                (ActionState::Ongoing, ActionState::Ongoing) => commands.trigger_targets(
                    ActionEvent::<A>::from(ActionEventKind::Ongoing {
                        value,
                        elapsed_secs: self.elapsed_secs,
                    }),
                    entity,
                ),
                (ActionState::Ongoing, ActionState::Fired) => commands.trigger_targets(
                    ActionEvent::<A>::from(ActionEventKind::Fired {
                        value,
                        fired_secs: self.fired_secs,
                        elapsed_secs: self.elapsed_secs,
                    }),
                    entity,
                ),
                (ActionState::Fired, ActionState::None) => commands.trigger_targets(
                    ActionEvent::<A>::from(ActionEventKind::Completed {
                        value,
                        fired_secs: self.fired_secs,
                        elapsed_secs: self.elapsed_secs,
                    }),
                    entity,
                ),
                (ActionState::Fired, ActionState::Ongoing) => commands.trigger_targets(
                    ActionEvent::<A>::from(ActionEventKind::Ongoing {
                        value,
                        elapsed_secs: self.elapsed_secs,
                    }),
                    entity,
                ),
                (ActionState::Fired, ActionState::Fired) => commands.trigger_targets(
                    ActionEvent::<A>::from(ActionEventKind::Fired {
                        value,
                        fired_secs: self.fired_secs,
                        elapsed_secs: self.elapsed_secs,
                    }),
                    entity,
                ),
            }
        }
    }

    pub fn state(&self) -> ActionState {
        self.state
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed_secs
    }

    pub fn fired_secs(&self) -> f32 {
        self.fired_secs
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionState {
    /// Condition is not triggered.
    #[default]
    None,
    /// Condition has started triggering, but has not yet finished.
    ///
    /// For example, a time-based condition requires its state to be maintained over several frames.
    Ongoing,
    /// The condition has been met.
    Fired,
}

#[derive(Debug, Event, Deref)]
pub struct ActionEvent<A: InputAction> {
    pub marker: PhantomData<A>,
    #[deref]
    pub kind: ActionEventKind,
}

impl<A: InputAction> From<ActionEventKind> for ActionEvent<A> {
    fn from(kind: ActionEventKind) -> Self {
        Self {
            marker: PhantomData,
            kind,
        }
    }
}

#[derive(Debug, Event)]
pub enum ActionEventKind {
    /// Triggers every frame when an action state is [`ActionState::Fired`].
    Fired {
        value: ActionValue,

        /// Time that this action was in [`ActionState::Fired`] state.
        fired_secs: f32,

        /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
        elapsed_secs: f32,
    },

    /// Triggers when an action switches its state from [`ActionState::None`]
    /// to [`ActionState::Fired`] or [`ActionState::Ongoing`].
    Started { value: ActionValue },

    /// Triggers every frame when an action state is [`ActionState::Ongoing`].
    Ongoing {
        value: ActionValue,

        /// Time that this action was in [`ActionState::Ongoing`] state.
        elapsed_secs: f32,
    },

    /// Triggers when action switches its state from [`ActionState::Fired`] to [`ActionState::None`],
    Completed {
        value: ActionValue,

        /// Time that this action was in [`ActionState::Fired`] state.
        fired_secs: f32,

        /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
        elapsed_secs: f32,
    },

    /// Triggers when action switches its state from [`ActionState::Ongoing`] to [`ActionState::None`],
    Canceled {
        value: ActionValue,

        /// Time that this action was in [`ActionState::Ongoing`] state.
        elapsed_secs: f32,
    },
}

impl ActionEventKind {
    pub fn is_fired(&self) -> bool {
        matches!(self, ActionEventKind::Fired { .. })
    }

    pub fn is_started(&self) -> bool {
        matches!(self, ActionEventKind::Started { .. })
    }

    pub fn is_ongoing(&self) -> bool {
        matches!(self, ActionEventKind::Ongoing { .. })
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, ActionEventKind::Completed { .. })
    }

    pub fn is_canceled(&self) -> bool {
        matches!(self, ActionEventKind::Canceled { .. })
    }
}

pub trait InputAction: Debug + Send + Sync + 'static {
    const DIM: ActionValueDim;

    /// Specifies whether this action should swallow any inputs bound to it or
    /// allow them to pass through to affect lower-priority bound actions.
    ///
    /// By default is set to `true`.
    const CONSUMES_INPUT: bool = true;

    /// Associated accumulation behavior.
    ///
    /// By default set to [`Accumulation::MaxAbs`].
    const ACCUMULATION: Accumulation = Accumulation::MaxAbs;
}

/// Defines how the value of an [`InputAction`] is calculated when there are multiple mappings.
#[derive(Default, Clone, Copy, Debug)]
pub enum Accumulation {
    /// Take the value from the mapping with the highest absolute value.
    ///
    /// For example, given values of 0.5 and -1.5, the input action's value would be -1.0.
    #[default]
    MaxAbs,

    /// Cumulatively add the key values for each mapping.
    ///
    /// For example, given values of 0.5 and -0.3, the input action's value would be 0.2.
    ///
    /// Usually used for things like WASD movement, when you want pressing W and S to cancel each other out.
    Cumulative,
}
