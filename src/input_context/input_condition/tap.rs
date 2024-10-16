use bevy::prelude::*;

use super::{held_timer::HeldTimer, InputCondition, DEFAULT_ACTUATION};
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
    pub actuation: f32,

    held_timer: HeldTimer,
    actuated: bool,
}

impl Tap {
    #[must_use]
    pub fn new(release_time: f32) -> Self {
        Self {
            release_time,
            actuation: DEFAULT_ACTUATION,
            held_timer: Default::default(),
            actuated: false,
        }
    }

    #[must_use]
    pub fn with_actuation(mut self, actuation: f32) -> Self {
        self.actuation = actuation;
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
        self.actuated = value.is_actuated(self.actuation);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap() {
        let world = World::new();
        let actions_data = ActionsData::default();

        let mut condition = Tap::new(1.0);
        assert_eq!(
            condition.evaluate(&world, &actions_data, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&world, &actions_data, 1.0, 0.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&world, &actions_data, 0.0, 0.0.into()),
            ActionState::None,
        );
        assert_eq!(
            condition.evaluate(&world, &actions_data, 2.0, 1.0.into()),
            ActionState::None,
        );
    }
}
