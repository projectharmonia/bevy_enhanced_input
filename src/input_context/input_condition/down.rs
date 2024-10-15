use bevy::prelude::*;

use super::{primitives::Actuation, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData},
};

/// Returns [`ActionState::Fired`] when the input exceeds the actuation threshold.
#[derive(Default, Debug)]
pub struct Down {
    /// Trigger threshold.
    pub actuation: Actuation,
}

impl Down {
    #[must_use]
    pub fn new(actuation: Actuation) -> Self {
        Self { actuation }
    }
}

impl InputCondition for Down {
    fn evaluate(
        &mut self,
        _world: &World,
        _actions_data: &ActionsData,
        _delta: f32,
        value: ActionValue,
    ) -> ActionState {
        if self.actuation.is_actuated(value) {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}
