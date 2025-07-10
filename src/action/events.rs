use core::fmt::{self, Debug, Formatter};

use bevy::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Bitset with events triggered by updating [`ActionState`] for an action.
///
/// Stored inside [`Action`].
///
/// On transition, events will be triggered with dedicated types that correspond to bitflags.
///
/// Table of state transitions:
///
/// | Last state                  | New state                | Events                    |
/// | --------------------------- | ------------------------ | ------------------------- |
/// | [`ActionState::None`]       | [`ActionState::None`]    | No events                 |
/// | [`ActionState::None`]       | [`ActionState::Ongoing`] | [`Started`] + [`Ongoing`] |
/// | [`ActionState::None`]       | [`ActionState::Fired`]   | [`Started`] + [`Fired`]   |
/// | [`ActionState::Ongoing`]    | [`ActionState::None`]    | [`Canceled`]              |
/// | [`ActionState::Ongoing`]    | [`ActionState::Ongoing`] | [`Ongoing`]               |
/// | [`ActionState::Ongoing`]    | [`ActionState::Fired`]   | [`Fired`]                 |
/// | [`ActionState::Fired`]      | [`ActionState::Fired`]   | [`Fired`]                 |
/// | [`ActionState::Fired`]      | [`ActionState::Ongoing`] | [`Ongoing`]               |
/// | [`ActionState::Fired`]      | [`ActionState::None`]    | [`Completed`]             |
///
/// The meaning of each kind depends on the assigned [`InputCondition`]s. The events are
/// fired in the action evaluation order.
#[derive(
    Component, Reflect, Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy,
)]
pub struct ActionEvents(u8);

bitflags! {
    impl ActionEvents: u8 {
        /// Corresponds to [`Started`].
        const STARTED = 0b00000001;
        /// Corresponds to [`Ongoing`].
        const ONGOING = 0b00000010;
        /// Corresponds to [`Fired`].
        const FIRED = 0b00000100;
        /// Corresponds to [`Canceled`].
        const CANCELED = 0b00001000;
        /// Corresponds to [`Completed`].
        const COMPLETED = 0b00010000;
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

/// Triggers when an action switches its state from [`ActionState::None`]
/// to [`ActionState::Fired`] or [`ActionState::Ongoing`].
///
/// Fired before [`Fired`] and [`Ongoing`].
///
/// For example, with the [`Down`] condition, this event is triggered
/// only once per press. It will not trigger again until the key is
/// released and pressed again.
///
/// See [`ActionEvents`] for all transitions.
#[derive(Event)]
pub struct Started<A: InputAction> {
    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,
}

impl<A: InputAction> Debug for Started<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Started")
            .field("value", &self.value)
            .field("state", &self.state)
            .finish()
    }
}

impl<A: InputAction> Clone for Started<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Started<A> {}

/// Triggers every frame when an action state is [`ActionState::Ongoing`].
///
/// For example, with the [`Hold`] condition, this event is triggered
/// continuously while the user is holding down the button, until the
/// specified duration is reached.
///
/// See [`ActionEvents`] for all transitions.
#[derive(Event)]
pub struct Ongoing<A: InputAction> {
    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Ongoing`] state.
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Ongoing<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ongoing")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Ongoing<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Ongoing<A> {}

/// Triggers every frame when an action state is [`ActionState::Fired`].
///
/// For example, with the [`Down`] condition, this event is triggered
/// every frame the key is held down.
///
/// See [`ActionEvents`] for all transitions.
#[derive(Event)]
pub struct Fired<A: InputAction> {
    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Fired`] state.
    pub fired_secs: f32,

    /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Fired<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fired")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("fired_secs", &self.fired_secs)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Fired<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Fired<A> {}

/// Triggers when action switches its state from [`ActionState::Ongoing`] to [`ActionState::None`].
///
/// For example, with the [`Hold`] condition, this event is triggered
/// if the user releases the button before the hold duration is met.
///
/// See [`ActionEvents`] for all transitions.
#[derive(Event)]
pub struct Canceled<A: InputAction> {
    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Ongoing`] state.
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Canceled<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Canceled")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Canceled<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Canceled<A> {}

/// Triggers when action switches its state from [`ActionState::Fired`] to [`ActionState::None`].
///
/// For example, with the [`Down`] condition, this event is triggered
/// when the user releases the key.
///
/// See [`ActionEvents`] for all transitions.
#[derive(Event)]
pub struct Completed<A: InputAction> {
    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Fired`] state.
    pub fired_secs: f32,

    /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Completed<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Completed")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("fired_secs", &self.fired_secs)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Completed<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Completed<A> {}
