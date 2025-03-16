use bevy::prelude::*;

use super::{condition_timer::ConditionTimer, InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// each [`Self::interval`] seconds.
///
/// Note: [`Completed`](crate::events::Completed) only fires
/// when the repeat limit is reached or when input is released immediately after being triggered.
/// Otherwise, [`Canceled`](crate::events::Canceled) is fired when input is released.
#[derive(Clone, Copy, Debug)]
pub struct Pulse {
    /// Time in seconds between each triggering while input is held.
    pub interval: f32,

    /// Number of times the condition can be triggered (0 means no limit).
    pub trigger_limit: u32,

    /// Whether to trigger when the input first exceeds the actuation threshold or wait for the first interval.
    pub trigger_on_start: bool,

    /// Trigger threshold.
    pub actuation: f32,

    timer: ConditionTimer,

    trigger_count: u32,
}

impl Pulse {
    #[must_use]
    pub fn new(interval: f32) -> Self {
        Self {
            interval,
            trigger_limit: 0,
            trigger_on_start: true,
            trigger_count: 0,
            actuation: DEFAULT_ACTUATION,
            timer: Default::default(),
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

    /// Enables or disables time dilation.
    #[must_use]
    pub fn relative_speed(mut self, relative: bool) -> Self {
        self.timer.relative_speed = relative;
        self
    }
}

impl InputCondition for Pulse {
    fn evaluate(
        &mut self,
        _actions: &ActionsData,
        time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionState {
        if value.is_actuated(self.actuation) {
            self.timer.update(time);

            if self.trigger_limit == 0 || self.trigger_count < self.trigger_limit {
                let trigger_count = if self.trigger_on_start {
                    self.trigger_count
                } else {
                    self.trigger_count + 1
                };

                // If the repeat count limit has not been reached.
                if self.timer.duration() >= self.interval * trigger_count as f32 {
                    // Trigger when held duration exceeds the interval threshold.
                    self.trigger_count += 1;
                    ActionState::Fired
                } else {
                    ActionState::Ongoing
                }
            } else {
                ActionState::None
            }
        } else {
            self.timer.reset();

            self.trigger_count = 0;
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::input_context::ActionsData;

    #[test]
    fn tap() {
        let mut condition = Pulse::new(1.0);
        let actions = ActionsData::default();
        let mut time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        time.advance_by(Duration::from_millis(500));
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        time.advance_by(Duration::ZERO);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn not_trigger_on_start() {
        let mut condition = Pulse::new(1.0).trigger_on_start(false);
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn trigger_limit() {
        let mut condition = Pulse::new(1.0).with_trigger_limit(1);
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None
        );
    }
}
