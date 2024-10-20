use super::{InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::{context_instance::ActionContext, input_action::ActionState},
};

/// Returns [`ActionState::Ongoing`]` when the input exceeds the actuation threshold and
/// [`ActionState::Fired`] once when the input drops back below the actuation threshold.
#[derive(Clone, Copy, Debug)]
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
    fn evaluate(&mut self, _ctx: &ActionContext, _delta: f32, value: ActionValue) -> ActionState {
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
    use bevy::prelude::*;

    use super::*;
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn released() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut condition = Released::default();
        assert_eq!(condition.evaluate(&ctx, 0.0, 0.0.into()), ActionState::None);
        assert_eq!(
            condition.evaluate(&ctx, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&ctx, 0.0, 0.0.into()),
            ActionState::Fired,
        );
    }
}
