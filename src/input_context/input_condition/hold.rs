use super::{condition_timer::ConditionTimer, InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::{context_instance::ActionContext, input_action::ActionState},
};

/// Returns [`ActionState::Ongoing`] when the input becomes actuated and
/// [`ActionState::Fired`] when input remained actuated for [`Self::hold_time`] seconds.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than [`Self::hold_time`] seconds.
/// May optionally fire once, or repeatedly fire.
#[derive(Clone, Copy, Debug)]
pub struct Hold {
    // How long does the input have to be held to cause trigger.
    pub hold_time: f32,

    // Should this trigger fire only once, or fire every frame once the hold time threshold is met?
    pub one_shot: bool,

    /// Trigger threshold.
    pub actuation: f32,

    timer: ConditionTimer,

    fired: bool,
}

impl Hold {
    #[must_use]
    pub fn new(hold_time: f32) -> Self {
        Self {
            hold_time,
            one_shot: false,
            actuation: DEFAULT_ACTUATION,
            timer: Default::default(),
            fired: false,
        }
    }

    #[must_use]
    pub fn one_shot(mut self, one_shot: bool) -> Self {
        self.one_shot = one_shot;
        self
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

impl InputCondition for Hold {
    fn evaluate(&mut self, ctx: &ActionContext, delta: f32, value: ActionValue) -> ActionState {
        let actuated = value.is_actuated(self.actuation);
        if actuated {
            self.timer.update(ctx.world, delta);
        } else {
            self.timer.reset();
        }

        let is_first_trigger = !self.fired;
        self.fired = self.timer.duration() >= self.hold_time;

        if self.fired {
            if is_first_trigger || !self.one_shot {
                ActionState::Fired
            } else {
                ActionState::None
            }
        } else if actuated {
            ActionState::Ongoing
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
    fn hold() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut condition = Hold::new(1.0);
        assert_eq!(
            condition.evaluate(&ctx, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&ctx, 1.0, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&ctx, 1.0, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(condition.evaluate(&ctx, 1.0, 0.0.into()), ActionState::None);
        assert_eq!(
            condition.evaluate(&ctx, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn one_shot() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut hold = Hold::new(1.0).one_shot(true);
        assert_eq!(hold.evaluate(&ctx, 1.0, 1.0.into()), ActionState::Fired);
        assert_eq!(hold.evaluate(&ctx, 1.0, 1.0.into()), ActionState::None);
    }
}
