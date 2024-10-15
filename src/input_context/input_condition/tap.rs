use bevy::prelude::*;

use super::{
    primitives::{Actuation, HeldTimer},
    InputCondition,
};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released within the [`Self::release_time`] seconds.
///
/// Returns [`ActionState::None`] when the input is actuated more than [`Self::release_time`] seconds.
#[derive(Debug)]
pub struct Tap {
    /// Time window within which the action must be released to register as a tap.
    pub release_time: f32,

    /// Trigger threshold.
    pub actuation: Actuation,

    held_timer: HeldTimer,
    actuated: bool,
}

impl Tap {
    #[must_use]
    pub fn new(release_time: f32) -> Self {
        Self {
            release_time,
            actuation: Default::default(),
            held_timer: Default::default(),
            actuated: false,
        }
    }

    #[must_use]
    pub fn with_actuation(mut self, actuation: impl Into<Actuation>) -> Self {
        self.actuation = actuation.into();
        self
    }

    #[must_use]
    pub fn with_held_timer(mut self, held_timer: HeldTimer) -> Self {
        self.held_timer = held_timer;
        self
    }
}

impl InputCondition for Tap {
    fn evaluate(
        &mut self,
        world: &World,
        _actions_data: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let last_actuated = self.actuated;
        let last_held_duration = self.held_timer.duration();
        self.actuated = self.actuation.is_actuated(value);
        if self.actuated {
            self.held_timer.update(world, delta);
        } else {
            self.held_timer.reset();
        }

        if last_actuated && !self.actuated && last_held_duration <= self.release_time {
            // Only trigger if pressed then released quickly enough.
            ActionState::Fired
        } else if self.held_timer.duration() >= self.release_time {
            // Once we pass the threshold halt all triggering until released.
            ActionState::None
        } else if self.actuated {
            ActionState::Ongoing
        } else {
            ActionState::None
        }
    }
}
