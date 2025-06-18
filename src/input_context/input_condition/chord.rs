use core::{
    any::{self, TypeId},
    marker::PhantomData,
};

use bevy::utils::TypeIdMap;
use log::warn;

use crate::prelude::*;

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
        action_map: &TypeIdMap<Action>,
        _time: &InputTime,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(action) = action_map.get(&TypeId::of::<A>()) {
            // Inherit state from the chorded action.
            action.state()
        } else {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!(
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
    use core::any::TypeId;

    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::input_time;

    #[test]
    fn chord() {
        let mut condition = Chord::<TestAction>::default();
        let mut action = Action::new::<TestAction>();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        action.update(&time, ActionState::Fired, true);
        let mut action_map = TypeIdMap::<Action>::default();
        action_map.insert(TypeId::of::<TestAction>(), action);

        assert_eq!(
            condition.evaluate(&action_map, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn missing_action() {
        let mut condition = Chord::<TestAction>::default();
        let action_map = TypeIdMap::<Action>::default();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(
            condition.evaluate(&action_map, &time, true.into()),
            ActionState::None,
        );
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct TestAction;
}
