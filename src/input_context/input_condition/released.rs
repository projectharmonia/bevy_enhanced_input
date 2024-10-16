use bevy::prelude::*;

use super::{InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`]` when the input exceeds the actuation threshold and
/// [`ActionState::Fired`] once when the input drops back below the actuation threshold.
#[derive(Debug)]
pub struct Released {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Released {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Released {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Released {
    fn evaluate(
        &mut self,
        _world: &World,
        _actions_data: &ActionsData,
        _delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let previosly_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated {
            // Ongoing on hold.
            ActionState::Ongoing
        } else if previosly_actuated {
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
    fn released() {
        let world = World::new();
        let actions_data = ActionsData::default();

        let mut released = Released::default();
        assert_eq!(
            released.evaluate(&world, &actions_data, 0.0, 0.0.into()),
            ActionState::None,
        );
        assert_eq!(
            released.evaluate(&world, &actions_data, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            released.evaluate(&world, &actions_data, 0.0, 0.0.into()),
            ActionState::Fired,
        );
    }
}
