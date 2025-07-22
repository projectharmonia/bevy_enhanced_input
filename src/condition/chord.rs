use bevy::prelude::*;
use log::warn;
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/// Returns the maximum [`ActionState`] among the given actions.
///
/// Useful for defining a composite action that fires only when all listed actions are active.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Chord {
    /// Actions whose state will be inherited when they are firing.
    pub actions: SmallVec<[Entity; 2]>,
}

impl Chord {
    /// Creates a new instance for a single action.
    #[must_use]
    pub fn single(action: Entity) -> Self {
        Self::new(smallvec![action])
    }

    /// Creates a new instance for multiple actions.
    #[must_use]
    pub fn new(actions: impl Into<SmallVec<[Entity; 2]>>) -> Self {
        Self {
            actions: actions.into(),
        }
    }
}

impl InputCondition for Chord {
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        _time: &ContextTime,
        _value: ActionValue,
    ) -> ActionState {
        // Inherit state from the most significant chorded action.
        let mut max_state = Default::default();
        for &action in &self.actions {
            let Ok((_, &state, ..)) = actions.get(action) else {
                // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
                warn!("`{action}` is not a valid action");
                continue;
            };

            if state > max_state {
                max_state = state;
            }
        }

        max_state
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::context;

    #[test]
    fn chord() {
        let (mut world, mut state) = context::init_world();
        let action = world
            .spawn((Action::<TestAction>::new(), ActionState::Fired))
            .id();
        let (time, actions) = state.get(&world);

        let mut condition = Chord::single(action);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Chord::single(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::None,
        );
    }

    #[derive(InputAction)]
    #[action_output(bool)]
    struct TestAction;
}
