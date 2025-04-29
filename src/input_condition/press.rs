use bevy::prelude::*;

use super::{DEFAULT_ACTUATION, InputCondition};
use crate::{
    action_map::{ActionMap, ActionState},
    action_value::ActionValue,
};

/// Returns [`ActionState::Fired`] when the input exceeds the actuation threshold.
#[derive(Clone, Copy, Debug)]
pub struct Down {
    /// Trigger threshold.
    pub actuation: f32,
}

impl Down {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self { actuation }
    }
}

impl Default for Down {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Down {
    fn evaluate(
        &mut self,
        _action_map: &ActionMap,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionState {
        if value.is_actuated(self.actuation) {
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
    fn down() {
        let mut condition = Down::new(1.0);
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
