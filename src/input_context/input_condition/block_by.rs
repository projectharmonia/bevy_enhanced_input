use std::{any, marker::PhantomData};

use bevy::prelude::*;

use super::{ConditionKind, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionState, ActionsData, InputAction},
};

/// Requires another action to not be fired within the same context.
#[derive(Debug)]
pub struct BlockBy<A: InputAction> {
    /// Action that blocks this condition when active.
    marker: PhantomData<A>,

    /// Wheter to block the state or only the events.
    ///
    /// By default set to false.
    pub events_only: bool,
}

impl<A: InputAction> BlockBy<A> {
    /// Block only events.
    ///
    /// For details, see [`ConditionKind::Blocker::events_only`].
    ///
    /// This can be used for chords to avoid triggering required actions.
    /// Otherwise, the chord will register the release and cancel itself.
    pub fn events_only() -> Self {
        Self {
            marker: PhantomData,
            events_only: true,
        }
    }
}

impl<A: InputAction> Default for BlockBy<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            events_only: false,
        }
    }
}

impl<A: InputAction> Clone for BlockBy<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for BlockBy<A> {}

impl<A: InputAction> InputCondition for BlockBy<A> {
    fn evaluate(
        &mut self,
        actions: &ActionsData,
        _time: &Time<Virtual>,
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
        ConditionKind::Blocker {
            events_only: self.events_only,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::input_context::input_action::ActionData;

    #[test]
    fn block() {
        let mut condition = BlockBy::<DummyAction>::default();
        let mut action = ActionData::new::<DummyAction>();
        let time = Time::default();
        action.update(&time, ActionState::Fired, true);
        let mut actions = ActionsData::default();
        actions.insert_action::<DummyAction>(action);

        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::None,
        );
    }

    #[test]
    fn missing_action() {
        let mut condition = BlockBy::<DummyAction>::default();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;
}
