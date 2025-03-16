use bevy::prelude::*;

use super::{InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`]` when the input exceeds the actuation threshold and
/// [`ActionState::Fired`] once when the input drops back below the actuation threshold.
#[derive(Clone, Copy, Debug)]
pub struct Release {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Release {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Release {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Release {
    fn evaluate(
        &mut self,
        _actions: &ActionsData,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated {
            // Ongoing on hold.
            ActionState::Ongoing
        } else if previously_actuated {
            // Fired on release.
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release() {
        let mut condition = Release::default();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
    }
}
