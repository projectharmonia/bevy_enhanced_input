use bevy::prelude::*;

use super::{primitives::Actuation, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Like [`super::down::Down`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Default, Debug)]
pub struct Pressed {
    /// Trigger threshold.
    pub actuation: Actuation,
    actuated: bool,
}

impl Pressed {
    #[must_use]
    pub fn new(actuation: Actuation) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl InputCondition for Pressed {
    fn evaluate(
        &mut self,
        _world: &World,
        _actions_data: &ActionsData,
        _delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let previosly_actuated = self.actuated;
        self.actuated = self.actuation.is_actuated(value);

        if self.actuated && !previosly_actuated {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}
