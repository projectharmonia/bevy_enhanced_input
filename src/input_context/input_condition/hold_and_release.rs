use bevy::prelude::*;

use super::{held_timer::HeldTimer, InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released after having been actuated for [`Self::hold_time`] seconds.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than [`Self::hold_time`] seconds.
#[derive(Debug)]
pub struct HoldAndRelease {
    // How long does the input have to be held to cause trigger.
    pub hold_time: f32,

    /// Trigger threshold.
    pub actuation: f32,

    held_timer: HeldTimer,
}

impl HoldAndRelease {
    #[must_use]
    pub fn new(hold_time: f32) -> Self {
        Self {
            hold_time,
            actuation: DEFAULT_ACTUATION,
            held_timer: Default::default(),
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

impl InputCondition for HoldAndRelease {
    fn evaluate(
        &mut self,
        world: &World,
        _actions: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        // Evaluate the updated held duration prior to checking for actuation.
        // This stops us failing to trigger if the input is released on the
        // threshold frame due to held duration being 0.
        self.held_timer.update(world, delta);
        let held_duration = self.held_timer.duration();

        if value.is_actuated(self.actuation) {
            ActionState::Ongoing
        } else {
            self.held_timer.reset();
            // Trigger if we've passed the threshold and released.
            if held_duration >= self.hold_time {
                ActionState::Fired
            } else {
                ActionState::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_and_release() {
        let world = World::new();
        let actions = ActionsData::default();

        let mut modifier = HoldAndRelease::new(1.0);
        assert_eq!(
            modifier.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            modifier.evaluate(&world, &actions, 1.0, 0.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            modifier.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            modifier.evaluate(&world, &actions, 0.0, 0.0.into()),
            ActionState::None,
        );
    }
}
