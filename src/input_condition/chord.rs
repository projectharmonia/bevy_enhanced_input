use core::{any, marker::PhantomData};

use bevy::prelude::*;

use super::{ConditionKind, InputCondition};
use crate::{
    action_map::{ActionMap, ActionState},
    action_value::ActionValue,
    input_action::InputAction,
};

/// Requires action `A` to be fired within the same context.
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
    fn evaluate(
        &mut self,
        action_map: &ActionMap,
        _time: &Time<Virtual>,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(action) = action_map.action::<A>() {
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
        ConditionKind::Implicit
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::action_map::{Action, ActionMap};

    #[test]
    fn chord() {
        let mut condition = Chord::<DummyAction>::default();
        let mut action = Action::new::<DummyAction>();
        let time = Time::default();
        action.update(&time, ActionState::Fired, true);
        let mut actions = ActionMap::default();
        actions.insert_action::<DummyAction>(action);

        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn missing_action() {
        let mut condition = Chord::<DummyAction>::default();
        let actions = ActionMap::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::None,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;
}
