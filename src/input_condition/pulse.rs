use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::{action_map::ActionMap, prelude::*};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// on the defined time interval.
///
/// Note: [`Completed`] only fires when the repeat limit is reached or when input is released
/// immediately after being triggered. Otherwise, [`Canceled`] is fired when input is released.
#[derive(Clone, Debug)]
pub struct Pulse {
    /// Number of times the condition can be triggered (0 means no limit).
    pub trigger_limit: u32,

    /// Whether to trigger when the input first exceeds the actuation threshold or wait for the first interval.
    pub trigger_on_start: bool,

    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,

    trigger_count: u32,
}

impl Pulse {
    /// Creates a new instance with the given interval in seconds.
    #[must_use]
    pub fn new(interval: f32) -> Self {
        Self {
            trigger_limit: 0,
            trigger_on_start: true,
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
            trigger_count: 0,
        }
    }

    #[must_use]
    pub fn with_trigger_limit(mut self, trigger_limit: u32) -> Self {
        self.trigger_limit = trigger_limit;
        self
    }

    #[must_use]
    pub fn trigger_on_start(mut self, trigger_on_start: bool) -> Self {
        self.trigger_on_start = trigger_on_start;
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

impl InputCondition for Pulse {
    fn evaluate(
        &mut self,
        _action_map: &ActionMap,
        time: &InputTime,
        value: ActionValue,
    ) -> ActionState {
        if value.is_actuated(self.actuation) {
            self.timer.tick(time.delta_kind(self.time_kind));

            if self.trigger_limit == 0 || self.trigger_count < self.trigger_limit {
                if self.timer.just_finished() {
                    self.trigger_count += self.timer.times_finished_this_tick();
                    ActionState::Fired
                } else {
                    ActionState::Ongoing
                }
            } else {
                ActionState::None
            }
        } else {
            self.timer.reset();
            if self.trigger_on_start {
                // Mock the timer to be triggered.
                self.timer.tick(self.timer.duration());
            }

            self.trigger_count = 0;
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
        let mut condition = Pulse::new(1.0);
        let action_map = ActionMap::default();
        let (mut world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired,
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(500));
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired,
        );

        world.resource_mut::<Time>().advance_by(Duration::ZERO);
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn not_trigger_on_start() {
        let mut condition = Pulse::new(1.0).trigger_on_start(false);
        let action_map = ActionMap::default();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn trigger_limit() {
        let mut condition = Pulse::new(1.0).with_trigger_limit(1);
        let action_map = ActionMap::default();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::None
        );
    }
}
