use bevy::prelude::*;

use super::{held_timer::HeldTimer, InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`] when the input becomes actuated and
/// [`ActionState::Fired`] when input remained actuated for [`Self::hold_time`] seconds.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than [`Self::hold_time`] seconds.
/// May optionally fire once, or repeatedly fire.
#[derive(Debug)]
pub struct Hold {
    // How long does the input have to be held to cause trigger.
    pub hold_time: f32,

    // Should this trigger fire only once, or fire every frame once the hold time threshold is met?
    pub one_shot: bool,

    /// Trigger threshold.
    pub actuation: f32,

    held_timer: HeldTimer,

    fired: bool,
}

impl Hold {
    #[must_use]
    pub fn new(hold_time: f32) -> Self {
        Self {
            hold_time,
            one_shot: false,
            actuation: DEFAULT_ACTUATION,
            held_timer: Default::default(),
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

    #[must_use]
    pub fn with_held_timer(mut self, held_timer: HeldTimer) -> Self {
        self.held_timer = held_timer;
        self
    }
}

impl InputCondition for Hold {
    fn evaluate(
        &mut self,
        world: &World,
        _actions_data: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let actuated = value.is_actuated(self.actuation);
        if actuated {
            self.held_timer.update(world, delta);
        } else {
            self.held_timer.reset();
        }

        let is_first_trigger = !self.fired;
        self.fired = self.held_timer.duration() >= self.hold_time;

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
    use super::*;

    #[test]
    fn hold() {
        let world = World::new();
        let actions_data = ActionsData::default();

        let mut condition = Hold::new(1.0);
        assert_eq!(
            condition.evaluate(&world, &actions_data, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&world, &actions_data, 1.0, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&world, &actions_data, 1.0, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&world, &actions_data, 1.0, 0.0.into()),
            ActionState::None,
        );
        assert_eq!(
            condition.evaluate(&world, &actions_data, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn one_shot() {
        let world = World::new();
        let actions_data = ActionsData::default();

        let mut hold = Hold::new(1.0).one_shot(true);
        assert_eq!(
            hold.evaluate(&world, &actions_data, 1.0, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            hold.evaluate(&world, &actions_data, 1.0, 1.0.into()),
            ActionState::None,
        );
    }
}
