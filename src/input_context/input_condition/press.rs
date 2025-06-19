use bevy::utils::TypeIdMap;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Like [`super::press::Down`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Clone, Copy, Debug)]
pub struct Press {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Press {
    #[must_use]
    pub fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Press {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Press {
    fn evaluate(
        &mut self,
        _action_map: &TypeIdMap<UntypedAction>,
        _time: &InputTime,
        value: ActionValue,
    ) -> ActionState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated && !previously_actuated {
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
    fn press() {
        let mut condition = Press::default();
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
