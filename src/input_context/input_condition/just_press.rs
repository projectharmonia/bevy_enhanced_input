use bevy::prelude::*;

use super::{InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Like [`super::press::Press`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Clone, Copy, Debug)]
pub struct JustPress {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl JustPress {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for JustPress {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for JustPress {
    fn evaluate(
        &mut self,
        _actions: &ActionsData,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionState {
        let previosly_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated && !previosly_actuated {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn press() {
        let mut condition = JustPress::default();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );
    }
}
