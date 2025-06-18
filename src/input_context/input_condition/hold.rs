use bevy::{prelude::*, utils::TypeIdMap};

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`ActionState::Ongoing`] when the input becomes actuated and
/// [`ActionState::Fired`] when input remained actuated for the defined hold time.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than the defined hold time.
/// May optionally fire once, or repeatedly fire.
#[derive(Clone, Debug)]
pub struct Hold {
    /// Should this trigger fire only once, or fire every frame once the hold time threshold is met?
    pub one_shot: bool,

    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,
}

impl Hold {
    /// Creates a new instance with the given hold time in seconds.
    #[must_use]
    pub fn new(hold_time: f32) -> Self {
        Self {
            one_shot: false,
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
            timer: Timer::from_seconds(hold_time, TimerMode::Once),
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
    pub fn with_time_kind(mut self, kind: TimeKind) -> Self {
        self.time_kind = kind;
        self
    }
}

impl InputCondition for Hold {
    fn evaluate(
        &mut self,
        _action_map: &TypeIdMap<Action>,
        time: &InputTime,
        value: ActionValue,
    ) -> ActionState {
        let actuated = value.is_actuated(self.actuation);
        if actuated {
            self.timer.tick(time.delta_kind(self.time_kind));
        } else {
            self.timer.reset();
        }

        if self.timer.finished() {
            if self.timer.just_finished() || !self.one_shot {
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
    use core::time::Duration;

    use super::*;
    use crate::input_time;

    #[test]
    fn hold() {
        let mut condition = Hold::new(1.0);
        let action_map = TypeIdMap::<Action>::default();
        let (mut world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Ongoing,
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::None
        );

        world.resource_mut::<Time>().advance_by(Duration::ZERO);
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn one_shot() {
        let mut hold = Hold::new(1.0).one_shot(true);
        let action_map = TypeIdMap::<Action>::default();
        let (mut world, mut state) = input_time::init_world();
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        let time = state.get(&world);

        assert_eq!(
            hold.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired
        );
        assert_eq!(
            hold.evaluate(&action_map, &time, 1.0.into()),
            ActionState::None
        );
    }
}
