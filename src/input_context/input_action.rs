use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use bevy::{prelude::*, utils::HashMap};
use bitflags::bitflags;

use crate::action_value::{ActionValue, ActionValueDim};

/// Map for actions to their [`ActionData`].
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
            value: ActionValue::zero(A::DIM),
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
                let event = ActionEvent::<A>::new(kind, self.value, self.state);
                trace!("triggering `{event:?}` for `{entity}`");
                commands.trigger_targets(event, entity);
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
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// fn move_character(trigger: Trigger<ActionEvent<Move>>) {
///    let event = trigger.event();
///
///    // The event implements `Deref` on `kind` for convenience:
///    if let ActionEventKind::Fired { fired_secs, elapsed_secs } = **event {
///        // ..
///    }
///
///    // You cal also use `is_*` helpers:
///    if event.is_fired() {
///        let movement = event.value.as_axis3d();
///        // ..
///    }
/// }
/// # #[derive(Debug, InputAction)]
/// # #[input_action(dim = Axis2D)]
/// # struct Move;
/// ```
#[derive(Debug, Event, Deref)]
pub struct ActionEvent<A: InputAction> {
    /// Action for which the event triggers.
    marker: PhantomData<A>,

    /// Type of [`ActionState`] transition.
    #[deref]
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

bitflags! {
    /// [`ActionEventKind`]s triggered for an action.
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ActionEvents: u8 {
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
    pub fn new(previous: ActionState, current: ActionState) -> ActionEvents {
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
    /// Inputs are consumed only if the action state is not equal to [`ActionState::None`].
    /// For details, see [`ContextInstance`](super::context_instance::ContextInstance).
    ///
    /// Consuming is global and affect actions in all contexts.
    const CONSUME_INPUT: bool = true;

    /// Associated accumulation behavior.
    const ACCUMULATION: Accumulation = Accumulation::Cumulative;
}

/// Defines how [`ActionValue`] is calculated when multiple inputs are evaluated with the
/// same most significant [`ActionState`] (excluding [`ActionState::None`]).
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

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;

    #[test]
    fn none_none() {
        let events = transition(ActionState::None, ActionState::None);
        assert!(events.is_empty());
    }

    #[test]
    fn none_ongoing() {
        let events = transition(ActionState::None, ActionState::Ongoing);
        let [event1, event2] = events.try_into().unwrap();
        assert!(event1.is_started());
        assert!(event2.is_ongoing());
    }

    #[test]
    fn none_fired() {
        let events = transition(ActionState::None, ActionState::Fired);
        let [event1, event2] = events.try_into().unwrap();
        assert!(event1.is_started());
        assert!(event2.is_fired());
    }

    #[test]
    fn ongoing_none() {
        let events = transition(ActionState::Ongoing, ActionState::None);
        let [event] = events.try_into().unwrap();
        assert!(event.is_canceled());
    }

    #[test]
    fn ongoing_ongoing() {
        let events = transition(ActionState::Ongoing, ActionState::Ongoing);
        let [event] = events.try_into().unwrap();
        assert!(event.is_ongoing());
    }

    #[test]
    fn ongoing_fired() {
        let events = transition(ActionState::Ongoing, ActionState::Fired);
        let [event] = events.try_into().unwrap();
        assert!(event.is_fired());
    }

    #[test]
    fn fired_none() {
        let events = transition(ActionState::Fired, ActionState::None);
        let [event] = events.try_into().unwrap();
        assert!(event.is_completed());
    }

    #[test]
    fn fired_ongoing() {
        let events = transition(ActionState::Fired, ActionState::Ongoing);
        let [event] = events.try_into().unwrap();
        assert!(event.is_ongoing());
    }

    #[test]
    fn fired_fired() {
        let events = transition(ActionState::Fired, ActionState::Fired);
        let [event] = events.try_into().unwrap();
        assert!(event.is_fired());
    }

    fn transition(
        initial_state: ActionState,
        target_state: ActionState,
    ) -> Vec<ActionEvent<DummyAction>> {
        let time = Time::<Virtual>::default();
        let mut action = ActionData::new::<DummyAction>();
        action.state = initial_state;
        action.update(&time, target_state, true);

        let mut world = World::new();
        world.init_resource::<TriggeredEvents>();
        world.observe(
            |trigger: Trigger<ActionEvent<DummyAction>>, mut events: ResMut<TriggeredEvents>| {
                events.push(*trigger.event());
            },
        );

        action.trigger_events(&mut world.commands(), &[Entity::PLACEHOLDER]);
        world.flush();

        world.remove_resource::<TriggeredEvents>().unwrap().0
    }

    #[derive(Resource, Default, Deref, DerefMut)]
    struct TriggeredEvents(Vec<ActionEvent<DummyAction>>);

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct DummyAction;
}
