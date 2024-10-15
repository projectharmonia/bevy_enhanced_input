use std::{any, marker::PhantomData};

use bevy::prelude::*;

use super::{ConditionKind, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData, InputAction},
};

/// Requires action `A` to be triggered within the same context.
///
/// Inherits [`ActionState`] from the specified action.
#[derive(Debug)]
pub struct Chord<A: InputAction> {
    /// Required action.
    pub marker: PhantomData<A>,
}

impl<A: InputAction> Default for Chord<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<A: InputAction> InputCondition for Chord<A> {
    fn evaluate(
        &mut self,
        _world: &World,
        actions_data: &ActionsData,
        _delta: f32,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(data) = actions_data.get_action::<A>() {
            // Inherit state from the chorded action.
            data.state()
        } else {
            warn_once!(
                "action `{}` is not present in context",
                any::type_name::<A>()
            );
            ActionState::None
        }
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}
