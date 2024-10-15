use std::{any, marker::PhantomData};

use bevy::prelude::*;

use super::{ConditionKind, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData, InputAction},
};

/// Requires another action to not be triggered within the same context.
///
/// Could be used for chords to avoid triggering required actions.
#[derive(Debug)]
pub struct BlockedBy<A: InputAction> {
    /// Action that blocks this condition when active.
    pub marker: PhantomData<A>,
}

impl<A: InputAction> Default for BlockedBy<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<A: InputAction> InputCondition for BlockedBy<A> {
    fn evaluate(
        &mut self,
        _world: &World,
        actions_data: &ActionsData,
        _delta: f32,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(data) = actions_data.get_action::<A>() {
            if data.state() == ActionState::Fired {
                return ActionState::None;
            }
        } else {
            warn_once!(
                "action `{}` is not present in context",
                any::type_name::<A>()
            );
        }

        ActionState::Fired
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}
