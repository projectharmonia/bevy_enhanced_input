use bevy::prelude::*;

use super::{primitives::Actuation, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`]` when the input exceeds the actuation threshold and
/// [`ActionState::Fired`] once when the input drops back below the actuation threshold.
#[derive(Default, Debug)]
pub struct Released {
    /// Trigger threshold.
    pub actuation: Actuation,
    actuated: bool,
}

impl Released {
    #[must_use]
    pub fn new(actuation: Actuation) -> Self {
        Self {
            actuation,
            actuated: false,
        }
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
        self.actuated = self.actuation.is_actuated(value);

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
