use bevy::prelude::*;

use super::{DEFAULT_ACTUATION, InputCondition, condition_timer::ConditionTimer};
use crate::{
    action_value::ActionValue,
    input_context::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released within the [`Self::release_time`] seconds.
///
/// Returns [`ActionState::None`] when the input is actuated more than [`Self::release_time`] seconds.
#[derive(Clone, Copy, Debug)]
pub struct Tap {
    /// Time window within which the action must be released to register as a tap.
    pub release_time: f32,

    /// Trigger threshold.
    pub actuation: f32,

    timer: ConditionTimer,
    actuated: bool,
}

impl Tap {
    #[must_use]
    pub fn new(release_time: f32) -> Self {
        Self {
            release_time,
            actuation: DEFAULT_ACTUATION,
            timer: Default::default(),
            actuated: false,
        }
    }

    #[must_use]
    pub fn with_actuation(mut self, actuation: f32) -> Self {
        self.actuation = actuation;
        self
    }

    /// Enables or disables time dilation.
    #[must_use]
    pub fn relative_speed(mut self, relative: bool) -> Self {
        self.timer.relative_speed = relative;
        self
    }
}

impl InputCondition for Tap {
    fn evaluate(
        &mut self,
        _actions: &ActionsData,
        time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionState {
        let last_actuated = self.actuated;
        let last_held_duration = self.timer.duration();
        self.actuated = value.is_actuated(self.actuation);
        if self.actuated {
            self.timer.update(time);
        } else {
            self.timer.reset();
        }

        if last_actuated && !self.actuated && last_held_duration <= self.release_time {
            // Only trigger if pressed then released quickly enough.
            ActionState::Fired
        } else if self.timer.duration() >= self.release_time {
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
    use core::time::Duration;

    use super::*;

    #[test]
    fn tap() {
        let mut condition = Tap::new(1.0);
        let actions = ActionsData::default();
        let mut time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );

        time.advance_by(Duration::from_secs(1));
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired,
        );

        time.advance_by(Duration::ZERO);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );

        time.advance_by(Duration::from_secs(2));
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None
        );
    }
}
