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
    marker: PhantomData<A>,
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
        actions: &ActionsData,
        _delta: f32,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(action) = actions.action::<A>() {
            // Inherit state from the chorded action.
            action.state()
        } else {
            warn_once!(
                "action `{}` is not present in context",
                any::type_name::<A>()
            );
            ActionState::None
        }
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
    fn chord() {
        let mut world = World::new();
        let mut action = ActionData::new::<DummyAction>();
        action.update(&mut world.commands(), &[], ActionState::Fired, true, 0.0);
        let mut actions = ActionsData::default();
        actions.insert(TypeId::of::<DummyAction>(), action);

        let mut condition = Chord::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, true.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn missing_action() {
        let world = World::new();
        let actions = ActionsData::default();

        let mut condition = Chord::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&world, &actions, 0.0, true.into()),
            ActionState::None,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct DummyAction;
}
