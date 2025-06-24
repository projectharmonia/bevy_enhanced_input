use bevy::utils::TypeIdMap;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

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
    fn evaluate(
        &mut self,
        _action_map: &TypeIdMap<UntypedAction>,
        _time: &InputTime,
        value: ActionValue,
    ) -> ActionState {
        if value.is_actuated(self.actuation) {
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
    fn down() {
        let mut condition = Down::new(1.0);
        let action_map = TypeIdMap::<UntypedAction>::default();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(
            condition.evaluate(&action_map, &time, 1.0.into()),
            ActionState::Fired,
        );
    }
}
