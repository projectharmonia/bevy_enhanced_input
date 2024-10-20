use super::{InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::{context_instance::ActionContext, input_action::ActionState},
};

/// Like [`super::down::Down`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Clone, Copy, Debug)]
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
    fn evaluate(&mut self, _ctx: &ActionContext, _delta: f32, value: ActionValue) -> ActionState {
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
    use bevy::prelude::*;

    use super::*;
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn pressed() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut condition = Pressed::default();
        assert_eq!(condition.evaluate(&ctx, 0.0, 0.0.into()), ActionState::None);
        assert_eq!(
            condition.evaluate(&ctx, 0.0, 1.0.into()),
            ActionState::Fired,
        );
    }
}
