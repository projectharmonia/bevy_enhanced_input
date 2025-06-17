use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::{action_map::ActionMap, prelude::*};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released within the defined release time.
///
/// Returns [`ActionState::None`] when the input is actuated more than the defined release time.
#[derive(Clone, Debug)]
pub struct Tap {
    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,

    actuated: bool,
}

impl Tap {
    /// Creates a new instance with the given release time in seconds.
    #[must_use]
    pub fn new(release_time: f32) -> Self {
        Self {
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
            timer: Timer::from_seconds(release_time, TimerMode::Once),
            actuated: false,
        }
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

impl InputCondition for Tap {
    fn evaluate(
        &mut self,
        _action_map: &ActionMap,
        time: &InputTime,
        value: ActionValue,
    ) -> ActionState {
        let last_actuated = self.actuated;
        let finished = self.timer.finished();
        self.actuated = value.is_actuated(self.actuation);
        if self.actuated {
            self.timer.tick(time.delta_kind(self.time_kind));
        } else {
            self.timer.reset();
        }

        if last_actuated && !self.actuated && !finished {
            // Only trigger if pressed then released quickly enough.
            ActionState::Fired
        } else if self.timer.finished() {
            // Once we pass the threshold halt all triggering until released.
            ActionState::None
        } else if self.actuated {
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
    fn tap() {
        let mut condition = Tap::new(1.0);
        let action_map = ActionMap::default();
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
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::Fired,
        );

        world.resource_mut::<Time>().advance_by(Duration::ZERO);
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::None
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(2));
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::None
        );
    }
}
