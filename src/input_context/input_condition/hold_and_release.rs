use super::{condition_timer::ConditionTimer, InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::{context_instance::ActionContext, input_action::ActionState},
};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released after having been actuated for [`Self::hold_time`] seconds.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than [`Self::hold_time`] seconds.
#[derive(Clone, Copy, Debug)]
pub struct HoldAndRelease {
    // How long does the input have to be held to cause trigger.
    pub hold_time: f32,

    /// Trigger threshold.
    pub actuation: f32,

    timer: ConditionTimer,
}

impl HoldAndRelease {
    #[must_use]
    pub fn new(hold_time: f32) -> Self {
        Self {
            hold_time,
            actuation: DEFAULT_ACTUATION,
            timer: Default::default(),
        }
    }

    #[must_use]
    pub fn with_actuation(mut self, actuation: f32) -> Self {
        self.actuation = actuation;
        self
    }

    /// Enables or disables time dilation.
    #[must_use]
    pub fn relative_speed(mut self, relative: bool) -> Self {
        self.timer.relative_speed = relative;
        self
    }
}

impl InputCondition for HoldAndRelease {
    fn evaluate(&mut self, ctx: &ActionContext, delta: f32, value: ActionValue) -> ActionState {
        // Evaluate the updated held duration prior to checking for actuation.
        // This stops us failing to trigger if the input is released on the
        // threshold frame due to held duration being 0.
        self.timer.update(ctx.world, delta);
        let held_duration = self.timer.duration();

        if value.is_actuated(self.actuation) {
            ActionState::Ongoing
        } else {
            self.timer.reset();
            // Trigger if we've passed the threshold and released.
            if held_duration >= self.hold_time {
                ActionState::Fired
            } else {
                ActionState::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn hold_and_release() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut modifier = HoldAndRelease::new(1.0);
        assert_eq!(
            modifier.evaluate(&ctx, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(modifier.evaluate(&ctx, 1.0, 0.0.into()), ActionState::Fired);
        assert_eq!(
            modifier.evaluate(&ctx, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(modifier.evaluate(&ctx, 0.0, 0.0.into()), ActionState::None);
    }
}
