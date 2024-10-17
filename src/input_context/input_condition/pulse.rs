use bevy::prelude::*;

use super::{held_timer::HeldTimer, InputCondition, DEFAULT_ACTUATION};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// each [`Self::interval`] seconds.
///
/// Note: [`ActionEventKind::Completed`](crate::input_context::input_action::ActionEventKind::Completed) only fires
/// when the repeat limit is reached or when input is released immediately after being triggered.
/// Otherwise, [`ActionEventKind::Canceled`](crate::input_context::input_action::ActionEventKind::Canceled) is fired when input is released.
#[derive(Debug)]
pub struct Pulse {
    /// Time in seconds between each triggering while input is held.
    pub interval: f32,

    // Number of times the condition can be triggered (0 means no limit).
    pub trigger_limit: u32,

    /// Whether to trigger when the input first exceeds the actuation threshold or wait for the first interval.
    pub trigger_on_start: bool,

    /// Trigger threshold.
    pub actuation: f32,

    held_timer: HeldTimer,

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
            held_timer: Default::default(),
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
    pub fn with_held_timer(mut self, held_timer: HeldTimer) -> Self {
        self.held_timer = held_timer;
        self
    }
}

impl InputCondition for Pulse {
    fn evaluate(
        &mut self,
        world: &World,
        _actions: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        if value.is_actuated(self.actuation) {
            self.held_timer.update(world, delta);

            if self.trigger_limit == 0 || self.trigger_count < self.trigger_limit {
                let trigger_count = if self.trigger_on_start {
                    self.trigger_count
                } else {
                    self.trigger_count + 1
                };

                // If the repeat count limit has not been reached.
                if self.held_timer.duration() >= self.interval * trigger_count as f32 {
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
            self.held_timer.reset();

            self.trigger_count = 0;
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap() {
        let world = World::new();
        let actions = ActionsData::default();

        let mut condition = Pulse::new(1.0);
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&world, &actions, 0.5, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&world, &actions, 0.5, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 0.0.into()),
            ActionState::None,
        );
    }

    #[test]
    fn not_trigger_on_start() {
        let world = World::new();
        let actions = ActionsData::default();

        let mut condition = Pulse::new(1.0).trigger_on_start(false);
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn trigger_limit() {
        let world = World::new();
        let actions = ActionsData::default();

        let mut condition = Pulse::new(1.0).with_trigger_limit(1);
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, 1.0.into()),
            ActionState::None,
        );
    }
}
