use super::{InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::{context_instance::ActionContext, input_action::ActionState},
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
    fn evaluate(&mut self, _ctx: &ActionContext, _delta: f32, value: ActionValue) -> ActionState {
        if value.is_actuated(self.actuation) {
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
    fn down() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut condition = Down::new(1.0);
        assert_eq!(condition.evaluate(&ctx, 0.0, 0.0.into()), ActionState::None);
        assert_eq!(
            condition.evaluate(&ctx, 0.0, 1.0.into()),
            ActionState::Fired,
        );
    }
}
