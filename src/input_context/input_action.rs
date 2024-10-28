use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use bevy::{prelude::*, utils::HashMap};
use bitflags::bitflags;

use crate::action_value::{ActionValue, ActionValueDim};

/// Map for actions to their [`ActionData`].
#[derive(Default, Deref, DerefMut)]
pub struct ActionsData(HashMap<TypeId, ActionData>);

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

impl From<HashMap<TypeId, ActionData>> for ActionsData {
    fn from(value: HashMap<TypeId, ActionData>) -> Self {
        Self(value)
    }
}

/// Tracker for action state.
#[derive(Clone, Copy)]
pub struct ActionData {
    state: ActionState,
    elapsed_secs: f32,
    fired_secs: f32,
    trigger_events: fn(&Self, &mut Commands, &[Entity], ActionState, ActionValue),
}

impl ActionData {
    /// Creates a new instance that will trigger events
    /// for action `A`.
    #[must_use]
    pub fn new<A: InputAction>() -> Self {
        Self {
            state: Default::default(),
            elapsed_secs: 0.0,
            fired_secs: 0.0,
            trigger_events: Self::trigger::<A>,
        }
    }

    /// Updates internal state and triggers corresponding events.
    pub fn update(
        &mut self,
        commands: &mut Commands,
        time: &Time<Virtual>,
        entities: &[Entity],
        state: ActionState,
        value: impl Into<ActionValue>,
    ) {
        // Add time from the previous frame if needed
        // before triggering events.
        match self.state {
            ActionState::None => (),
            ActionState::Ongoing => {
                self.elapsed_secs += time.delta_seconds();
            }
            ActionState::Fired => {
                self.elapsed_secs += time.delta_seconds();
                self.fired_secs += time.delta_seconds();
            }
        }

        (self.trigger_events)(self, commands, entities, state, value.into());
        self.state = state;

        // Reset time for updated state.
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

    /// Trigger events for removed entities.
    ///
    /// This will trigger transition from the current state to [`ActionState::None`]
    /// simulating releasing the input.
    ///
    /// Internal state won't be updated.
    /// Called for removed entities for shared contexts or for context removals.
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
        let events = ActionEvents::new(self.state, state);
        for (_, event) in events.iter_names() {
            let kind = match event {
                ActionEvents::FIRED => ActionEventKind::Fired {
                    fired_secs: self.fired_secs,
                    elapsed_secs: self.elapsed_secs,
                },
                ActionEvents::STARTED => ActionEventKind::Started,
                ActionEvents::ONGOING => ActionEventKind::Ongoing {
                    elapsed_secs: self.elapsed_secs,
                },
                ActionEvents::COMPLETED => ActionEventKind::Completed {
                    fired_secs: self.fired_secs,
                    elapsed_secs: self.elapsed_secs,
                },
                ActionEvents::CANCELED => ActionEventKind::Canceled {
                    elapsed_secs: self.elapsed_secs,
                },
                _ => unreachable!("iteration should yield only named flags"),
            };

            // Trigger an event for each entity separately
            // since it's cheaper to copy the event than to clone the entities.
            for &entity in entities {
                let event = ActionEvent::<A>::new(kind, value, state);
                trace!("triggering `{event:?}` for `{entity}`");
                commands.trigger_targets(event, entity);
            }
        }
    }

    /// Returns the current state.
    pub fn state(&self) -> ActionState {
        self.state
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

/// State for [`ActionData`].
///
/// States are ordered by their significance.
///
/// See also [`ActionEvent`].
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

/// Trigger emitted for transitions between [`ActionState`]s for action `A`.
///
/// See also [`UntypedActionEvent`].
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// fn move_character(trigger: Trigger<ActionEvent<Move>>) {
///    let event = trigger.event();
///    if let ActionEventKind::Fired { fired_secs, elapsed_secs } = event.kind {
///        // ..
///    }
///
///    // You cal also use `is_*` helpers.
///    if event.kind.is_fired() {
///        // ..
///    }
/// }
/// # #[derive(Debug, InputAction)]
/// # #[input_action(dim = Axis2D)]
/// # struct Move;
/// ```
#[derive(Debug, Event)]
pub struct ActionEvent<A: InputAction> {
    /// Action for which the event triggers.
    marker: PhantomData<A>,

    /// Type of [`ActionState`] transition.
    pub kind: ActionEventKind,

    /// Current action value.
    pub value: ActionValue,

    /// Current action state.
    pub state: ActionState,
}

impl<A: InputAction> ActionEvent<A> {
    /// Creates a new event for `A`.
    #[must_use]
    pub fn new(kind: ActionEventKind, value: ActionValue, state: ActionState) -> Self {
        Self {
            marker: PhantomData,
            kind,
            value,
            state,
        }
    }
}

impl<A: InputAction> Clone for ActionEvent<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for ActionEvent<A> {}

impl<A: InputAction> From<UntypedActionEvent> for ActionEvent<A> {
    fn from(value: UntypedActionEvent) -> Self {
        Self {
            marker: PhantomData,
            kind: value.kind,
            value: value.value,
            state: value.state,
        }
    }
}

/// Represents an [`ActionEvent`] without its associated action type marker.
///
/// This can be used to store event data in a type-erased manner.
#[derive(Clone, Copy)]
pub struct UntypedActionEvent {
    pub kind: ActionEventKind,
    pub value: ActionValue,
    pub state: ActionState,
}

impl<A: InputAction> From<ActionEvent<A>> for UntypedActionEvent {
    fn from(value: ActionEvent<A>) -> Self {
        Self {
            kind: value.kind,
            value: value.value,
            state: value.state,
        }
    }
}

bitflags! {
    /// [`ActionEventKind`]s triggered for an action.
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    struct ActionEvents: u8 {
        /// Corresponds to [`ActionEventKind::Started`].
        const STARTED = 0b00000001;
        /// Corresponds to [`ActionEventKind::Fired`].
        const FIRED = 0b00000010;
        /// Corresponds to [`ActionEventKind::Ongoing`].
        const ONGOING = 0b00000100;
        /// Corresponds to [`ActionEventKind::Completed`].
        const COMPLETED = 0b00001000;
        /// Corresponds to [`ActionEventKind::Canceled`].
        const CANCELED = 0b00010000;
    }
}

impl ActionEvents {
    /// Creates a new instance based on state transition.
    fn new(previous: ActionState, current: ActionState) -> ActionEvents {
        match (previous, current) {
            (ActionState::None, ActionState::None) => ActionEvents::empty(),
            (ActionState::None, ActionState::Ongoing) => {
                ActionEvents::STARTED | ActionEvents::ONGOING
            }
            (ActionState::None, ActionState::Fired) => ActionEvents::STARTED | ActionEvents::FIRED,
            (ActionState::Ongoing, ActionState::None) => ActionEvents::CANCELED,
            (ActionState::Ongoing, ActionState::Ongoing) => ActionEvents::ONGOING,
            (ActionState::Ongoing, ActionState::Fired) => ActionEvents::FIRED,
            (ActionState::Fired, ActionState::None) => ActionEvents::COMPLETED,
            (ActionState::Fired, ActionState::Ongoing) => ActionEvents::ONGOING,
            (ActionState::Fired, ActionState::Fired) => ActionEvents::FIRED,
        }
    }
}

/// Represents the type of event triggered by updating [`ActionState`].
///
/// Table of state transitions:
///
/// | Last state                  | New state                | Events                                |
/// | --------------------------- | ------------------------ | ------------------------------------- |
/// | [`ActionState::None`]       | [`ActionState::None`]    | No events                             |
/// | [`ActionState::None`]       | [`ActionState::Ongoing`] | [`Self::Started`] + [`Self::Ongoing`] |
/// | [`ActionState::None`]       | [`ActionState::Fired`]   | [`Self::Started`] + [`Self::Fired`]   |
/// | [`ActionState::Ongoing`]    | [`ActionState::None`]    | [`Self::Canceled`]                    |
/// | [`ActionState::Ongoing`]    | [`ActionState::Ongoing`] | [`Self::Ongoing`]                     |
/// | [`ActionState::Ongoing`]    | [`ActionState::Fired`]   | [`Self::Fired`]                       |
/// | [`ActionState::Fired`]      | [`ActionState::Fired`]   | [`Self::Fired`]                       |
/// | [`ActionState::Fired`]      | [`ActionState::Ongoing`] | [`Self::Ongoing`]                     |
/// | [`ActionState::Fired`]      | [`ActionState::None`]    | [`Self::Completed`]                   |
///
/// The meaning of each kind depends on the assigned [`InputCondition`](super::input_condition::InputCondition)s.
#[derive(Debug, Event, Clone, Copy)]
pub enum ActionEventKind {
    /// Triggers every frame when an action state is [`ActionState::Fired`].
    ///
    /// For example, with the [`Release`](super::input_condition::release::Release) condition,
    /// this event is triggered when the user releases the key.
    Fired {
        /// Time that this action was in [`ActionState::Fired`] state.
        fired_secs: f32,

        /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
        elapsed_secs: f32,
    },
    /// Triggers when an action switches its state from [`ActionState::None`]
    /// to [`ActionState::Fired`] or [`ActionState::Ongoing`].
    ///
    /// Fired before [`Self::Fired`] and [`Self::Ongoing`].
    ///
    /// For example, with the [`Tap`](super::input_condition::tap::Tap) condition, this event is triggered
    /// only on the first press.
    Started,
    /// Triggers every frame when an action state is [`ActionState::Ongoing`].
    ///
    /// For example, with the [`HoldAndRelease`](super::input_condition::hold_and_release::HoldAndRelease) condition,
    /// this event is triggered while the user is holding down the button before the specified duration is reached.
    Ongoing {
        /// Time that this action was in [`ActionState::Ongoing`] state.
        elapsed_secs: f32,
    },
    /// Triggers when action switches its state from [`ActionState::Fired`] to [`ActionState::None`],
    ///
    /// For example, with the [`Hold`](super::input_condition::hold::Hold) condition,
    /// this event is triggered when the user releases the key.
    Completed {
        /// Time that this action was in [`ActionState::Fired`] state.
        fired_secs: f32,

        /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
        elapsed_secs: f32,
    },
    /// Triggers when action switches its state from [`ActionState::Ongoing`] to [`ActionState::None`],
    ///
    /// For example, with the [`HoldAndRelease`](super::input_condition::hold_and_release::HoldAndRelease) condition,
    /// this event is triggered if the user releases the button before the condition is triggered.
    Canceled {
        /// Time that this action was in [`ActionState::Ongoing`] state.
        elapsed_secs: f32,
    },
}

impl ActionEventKind {
    /// Returns `true` if the value is [`ActionEventKind::Fired`].
    #[must_use]
    pub fn is_fired(&self) -> bool {
        matches!(self, ActionEventKind::Fired { .. })
    }

    /// Returns `true` if the value is [`ActionEventKind::Started`].
    #[must_use]
    pub fn is_started(&self) -> bool {
        matches!(self, ActionEventKind::Started { .. })
    }

    /// Returns `true` if the value is [`ActionEventKind::Ongoing`].
    #[must_use]
    pub fn is_ongoing(&self) -> bool {
        matches!(self, ActionEventKind::Ongoing { .. })
    }

    /// Returns `true` if the value is [`ActionEventKind::Completed`].
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self, ActionEventKind::Completed { .. })
    }

    /// Returns `true` if the value is [`ActionEventKind::Canceled`].
    #[must_use]
    pub fn is_canceled(&self) -> bool {
        matches!(self, ActionEventKind::Canceled { .. })
    }
}

/// Marker for a gameplay-related action.
///
/// Needs to be bind inside
/// [`InputContext::context_instance`](super::InputContext::context_instance)
///
/// Each binded action will have [`ActionState`].
/// When it updates during [`ContextInstance`](super::context_instance::ContextInstance)
/// evaluation, an [`ActionEvent`] is triggered.
///
/// To implement the trait you can use the [`InputAction`](bevy_enhanced_input_macros::InputAction)
/// derive to reduce boilerplate:
///
/// ```
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(Debug, InputAction)]
/// #[input_action(dim = Axis2D)]
/// struct Move;
/// ```
///
/// Optionally you can pass `consume_input` and/or `accumulation`:
///
/// ```
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(Debug, InputAction)]
/// #[input_action(dim = Axis2D, accumulation = Cumulative, consume_input = false)]
/// struct Move;
/// ```
pub trait InputAction: Debug + Send + Sync + 'static {
    /// Discriminant for [`ActionValue`] that will be used for this action.
    ///
    /// Use [`ActionValueDim::Bool`] for button-like actions (e.g., `Jump`).
    /// Use [`ActionValueDim::Axis1D`] for single-axis actions (e.g., `Zoom`).
    /// For multi-axis actions, like `Move`, use [`ActionValueDim::Axis2D`] or [`ActionValueDim::Axis3D`].
    const DIM: ActionValueDim;

    /// Specifies whether this action should swallow any [`Input`](crate::input::Input)s
    /// bound to it or allow them to pass through to affect other actions.
    ///
    /// Inputs are consumed only if their [`Modifiers`](crate::input::Modifiers)
    /// are also pressed.
    ///
    /// Consuming is global and affect actions in all contexts.
    const CONSUME_INPUT: bool = true;

    /// Associated accumulation behavior.
    const ACCUMULATION: Accumulation = Accumulation::Cumulative;
}

/// Defines how [`ActionValue`] is calculated when multiple inputs are evaluated with the same [`ActionState`].
#[derive(Default, Clone, Copy, Debug)]
pub enum Accumulation {
    /// Cumulatively add the key values for each mapping.
    ///
    /// For example, given values of 0.5 and -0.3, the input action's value would be 0.2.
    ///
    /// Usually used for things like WASD movement, when you want pressing W and S to cancel each other out.
    #[default]
    Cumulative,
    /// Take the value from the mapping with the highest absolute value.
    ///
    /// For example, given values of 0.5 and -1.5, the input action's value would be -1.5.
    MaxAbs,
}
