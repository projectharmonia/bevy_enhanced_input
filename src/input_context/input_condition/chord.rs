use std::{any, marker::PhantomData};

use bevy::prelude::*;

use super::{ConditionKind, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::{
        context_instance::ActionContext,
        input_action::{ActionState, InputAction},
    },
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

impl<A: InputAction> Clone for Chord<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Chord<A> {}

impl<A: InputAction> InputCondition for Chord<A> {
    fn evaluate(&mut self, ctx: &ActionContext, _delta: f32, _value: ActionValue) -> ActionState {
        if let Some(action) = ctx.actions.action::<A>() {
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
    use crate::{
        input_context::input_action::{ActionData, ActionsData},
        ActionValueDim,
    };

    #[test]
    fn chord() {
        let mut world = World::new();
        let mut action = ActionData::new::<DummyAction>();
        action.update(&mut world.commands(), &[], ActionState::Fired, true, 0.0);
        let mut actions = ActionsData::default();
        actions.insert(TypeId::of::<DummyAction>(), action);
        let ctx = ActionContext {
            world: &world,
            actions: &actions,
            entities: &[],
        };

        let mut condition = Chord::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&ctx, 0.0, true.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn missing_action() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut condition = Chord::<DummyAction>::default();
        assert_eq!(
            condition.evaluate(&ctx, 0.0, true.into()),
            ActionState::None,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct DummyAction;
}
