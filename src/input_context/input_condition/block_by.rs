use core::{
    any::{self, TypeId},
    marker::PhantomData,
};

use bevy::utils::TypeIdMap;
use log::warn;

use crate::prelude::*;

/// Requires another action to not be fired within the same context.
#[derive(Debug)]
pub struct BlockBy<A: InputAction> {
    /// Action that blocks this condition when active.
    marker: PhantomData<A>,
}

impl<A: InputAction> Default for BlockBy<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
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
        action_map: &TypeIdMap<UntypedAction>,
        _time: &InputTime,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(action) = action_map.get(&TypeId::of::<A>()) {
            if action.state == ActionState::Fired {
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
        ConditionKind::Blocker
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::input_time;

    #[test]
    fn block() {
        let mut condition = BlockBy::<TestAction>::default();
        let mut action = UntypedAction::new::<TestAction>();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        action.update(&time, ActionState::Fired, true);
        let mut action_map = TypeIdMap::<UntypedAction>::default();
        action_map.insert(TypeId::of::<TestAction>(), action);

        assert_eq!(
            condition.evaluate(&action_map, &time, true.into()),
            ActionState::None,
        );
    }

    #[test]
    fn missing_action() {
        let mut condition = BlockBy::<TestAction>::default();
        let action_map = TypeIdMap::<UntypedAction>::default();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct TestAction;
}
