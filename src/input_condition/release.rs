use super::DEFAULT_ACTUATION;
use crate::{action_map::ActionMap, prelude::*};

/// Returns [`ActionState::Ongoing`] when the input exceeds the actuation threshold and
/// [`ActionState::Fired`] once when the input drops back below the actuation threshold.
#[derive(Clone, Copy, Debug)]
pub struct Release {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Release {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Release {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Release {
    fn evaluate(
        &mut self,
        _action_map: &ActionMap,
        _time: &InputTime,
        value: ActionValue,
    ) -> ActionState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated {
            // Ongoing on hold.
            ActionState::Ongoing
        } else if previously_actuated {
            // Fired on release.
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_time;

    #[test]
    fn release() {
        let mut condition = Release::default();
        let action_map = ActionMap::default();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::Fired
        );
    }
}
