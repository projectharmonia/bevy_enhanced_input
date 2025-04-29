use core::{any, marker::PhantomData};

use bevy::prelude::*;
use log::warn;

use super::InputModifier;
use crate::{
    InputAction,
    action_map::{ActionMap, ActionState},
    action_value::ActionValue,
};

/// Produces accumulated value when another action is fired within the same context.
///
/// Continuously adds input values together as long as action `A` is [`ActionState::Fired`].
/// When the action is inactive, it resets the accumulation with the current frame's input value.
#[derive(Clone, Copy, Debug)]
pub struct AccumulateBy<A: InputAction> {
    /// Action that activates accumulation.
    marker: PhantomData<A>,

    /// The accumulated value across frames.
    value: Vec3,
}

impl<A: InputAction> Default for AccumulateBy<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            value: Default::default(),
        }
    }
}

impl<A: InputAction> InputModifier for AccumulateBy<A> {
    fn apply(
        &mut self,
        action_map: &ActionMap,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        if let Some(action) = action_map.action::<A>() {
            if action.state() == ActionState::Fired {
                self.value += value.as_axis3d();
            } else {
                self.value = value.as_axis3d();
            }
            ActionValue::Axis3D(self.value).convert(value.dim())
        } else {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!(
                "action `{}` is not present in context",
                any::type_name::<A>()
            );
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::action_map::Action;

    #[test]
    fn accumulation_active() {
        let mut modifier = AccumulateBy::<TestAction>::default();
        let mut action = Action::new::<TestAction>();
        let time = Time::default();
        action.update(&time, ActionState::Fired, true);
        let mut action_map = ActionMap::default();
        action_map.insert_action::<TestAction>(action);

        assert_eq!(modifier.apply(&action_map, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&action_map, &time, 1.0.into()), 2.0.into());
    }

    #[test]
    fn accumulation_inactive() {
        let mut modifier = AccumulateBy::<TestAction>::default();
        let action = Action::new::<TestAction>();
        let time = Time::default();
        let mut action_map = ActionMap::default();
        action_map.insert_action::<TestAction>(action);

        assert_eq!(modifier.apply(&action_map, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&action_map, &time, 1.0.into()), 1.0.into());
    }

    #[test]
    fn missing_action() {
        let mut modifier = AccumulateBy::<TestAction>::default();
        let action_map = ActionMap::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&action_map, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&action_map, &time, 1.0.into()), 1.0.into());
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct TestAction;
}
