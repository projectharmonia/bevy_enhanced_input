use bevy::prelude::*;

use super::{DEFAULT_ACTUATION, InputCondition};
use crate::{
    action_map::{ActionMap, ActionState},
    action_value::ActionValue,
};

/// Like [`super::press::Down`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Clone, Copy, Debug)]
pub struct Press {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Press {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Press {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Press {
    fn evaluate(
        &mut self,
        _action_map: &ActionMap,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated && !previously_actuated {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_map::ActionMap;

    #[test]
    fn press() {
        let mut condition = Press::default();
        let action_map = ActionMap::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired,
        );
    }
}
