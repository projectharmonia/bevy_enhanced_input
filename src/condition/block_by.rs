use bevy::prelude::*;
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/// Requires another action to not be fired within the same context.
#[derive(Component, Reflect, Debug, Clone)]
pub struct BlockBy {
    /// Action that blocks this condition when active.
    pub actions: SmallVec<[Entity; 2]>,
}

impl BlockBy {
    pub fn single(action: Entity) -> Self {
        Self::new(smallvec![action])
    }

    pub fn new(actions: impl Into<SmallVec<[Entity; 2]>>) -> Self {
        Self {
            actions: actions.into(),
        }
    }
}

impl InputCondition for BlockBy {
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        _time: &ContextTime,
        _value: ActionValue,
    ) -> ActionState {
        if actions
            .iter_many(&self.actions)
            .all(|(_, &state, ..)| state == ActionState::Fired)
        {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Blocker
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::context;

    #[test]
    fn block() {
        let (mut world, mut state) = context::init_world();
        let action = world
            .spawn((Action::<TestAction>::new(), ActionState::Fired))
            .id();
        let (time, actions) = state.get(&world);

        let mut condition = BlockBy::single(action);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::None,
        );
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = BlockBy::single(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[derive(InputAction)]
    #[input_action(output = bool)]
    struct TestAction;
}
