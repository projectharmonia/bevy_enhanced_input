use bevy::prelude::*;

use super::{InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Like [`super::down::Down`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Debug)]
pub struct Pressed {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Pressed {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Pressed {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Pressed {
    fn evaluate(
        &mut self,
        _world: &World,
        _actions: &ActionsData,
        _delta: f32,
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

    #[test]
    fn pressed() {
        let world = World::new();
        let actions = ActionsData::default();

        let mut condition = Pressed::default();
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 0.0.into()),
            ActionState::None,
        );
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::Fired,
        );
    }
}
