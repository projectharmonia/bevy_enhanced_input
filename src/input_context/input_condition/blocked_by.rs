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
    marker: PhantomData<A>,
}

impl<A: InputAction> Default for BlockedBy<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<A: InputAction> Clone for BlockedBy<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for BlockedBy<A> {}

impl<A: InputAction> InputCondition for BlockedBy<A> {
    fn evaluate(
        &mut self,
        _world: &World,
        actions: &ActionsData,
        _delta: f32,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(action) = actions.action::<A>() {
            if action.state() == ActionState::Fired {
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
        ConditionKind::Required
    }
}

#[cfg(test)]
mod tests {
    use any::TypeId;

    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::{input_context::input_action::ActionData, ActionValueDim};

    #[test]
    fn blocked() {
        let mut world = World::new();
        let mut action = ActionData::new::<DummyAction>();
        action.update(&mut world.commands(), &[], ActionState::Fired, true, 0.0);
        let mut actions = ActionsData::default();
        actions.insert(TypeId::of::<DummyAction>(), action);

        let mut condition = BlockedBy::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, true.into()),
            ActionState::None,
        );
    }

    #[test]
    fn missing_action() {
        let world = World::new();
        let actions = ActionsData::default();

        let mut condition = BlockedBy::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, true.into()),
            ActionState::Fired,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct DummyAction;
}
