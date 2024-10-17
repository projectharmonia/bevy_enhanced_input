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

#[cfg(test)]
mod tests {
    use any::TypeId;

    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::{input_context::input_action::ActionData, ActionValueDim};

    #[test]
    fn chord() {
        let mut world = World::new();
        let mut data = ActionData::new::<DummyAction>();
        data.update(
            &mut world.commands(),
            &[],
            ActionState::Fired,
            true.into(),
            0.0,
        );
        let mut actions_data = ActionsData::default();
        actions_data.insert(TypeId::of::<DummyAction>(), data);

        let mut condition = Chord::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&world, &actions_data, 0.0, true.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn missing_action() {
        let world = World::new();
        let actions_data = ActionsData::default();

        let mut condition = Chord::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&world, &actions_data, 0.0, true.into()),
            ActionState::None,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct DummyAction;
}
