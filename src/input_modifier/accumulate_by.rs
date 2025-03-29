use core::{any, marker::PhantomData};

use bevy::prelude::*;

use super::InputModifier;
use crate::{
    InputAction,
    action_value::ActionValue,
    actions::{ActionState, ActionsData},
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
        actions: &ActionsData,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        if let Some(action) = actions.action::<A>() {
            if action.state() == ActionState::Fired {
                self.value += value.as_axis3d();
            } else {
                self.value = value.as_axis3d();
            }
            ActionValue::Axis3D(self.value).convert(value.dim())
        } else {
            warn_once!(
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
    use crate::actions::ActionData;

    #[test]
    fn accumulation_active() {
        let mut modifier = AccumulateBy::<DummyAction>::default();
        let mut action = ActionData::new::<DummyAction>();
        let time = Time::default();
        action.update(&time, ActionState::Fired, true);
        let mut actions = ActionsData::default();
        actions.insert_action::<DummyAction>(action);

        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 2.0.into());
    }

    #[test]
    fn accumulation_inactive() {
        let mut modifier = AccumulateBy::<DummyAction>::default();
        let action = ActionData::new::<DummyAction>();
        let time = Time::default();
        let mut actions = ActionsData::default();
        actions.insert_action::<DummyAction>(action);

        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());
    }

    #[test]
    fn missing_action() {
        let mut modifier = AccumulateBy::<DummyAction>::default();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;
}
