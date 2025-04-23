use core::{any, marker::PhantomData};

use bevy::prelude::*;
use log::warn;

use super::{ConditionKind, InputCondition};
use crate::{
    action_map::{ActionMap, ActionState},
    action_value::ActionValue,
    input_action::InputAction,
};

/// Requires another action to not be fired within the same context.
#[derive(Debug)]
pub struct BlockBy<A: InputAction> {
    /// Action that blocks this condition when active.
    marker: PhantomData<A>,

    /// Whether to block the state or only the events.
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
        action_map: &ActionMap,
        _time: &Time<Virtual>,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(action) = action_map.action::<A>() {
            if action.state() == ActionState::Fired {
                return ActionState::None;
            }
        } else {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!(
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
    use crate::action_map::Action;

    #[test]
    fn block() {
        let mut condition = BlockBy::<DummyAction>::default();
        let mut action = Action::new::<DummyAction>();
        let time = Time::default();
        action.update(&time, ActionState::Fired, true);
        let mut action_map = ActionMap::default();
        action_map.insert_action::<DummyAction>(action);

        assert_eq!(
            condition.evaluate(&action_map, &time, true.into()),
            ActionState::None,
        );
    }

    #[test]
    fn missing_action() {
        let mut condition = BlockBy::<DummyAction>::default();
        let action_map = ActionMap::default();
        let time = Time::default();

        assert_eq!(
            condition.evaluate(&action_map, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;
}
