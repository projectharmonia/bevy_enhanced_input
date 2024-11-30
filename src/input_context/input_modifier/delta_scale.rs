use bevy::prelude::*;

use super::InputModifier;
use crate::{action_value::ActionValue, input_context::context_instance::ActionsData};

/// Multiplies the input value by delta time for this frame.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Clone, Copy, Debug)]
pub struct DeltaScale;

impl InputModifier for DeltaScale {
    fn apply(
        &mut self,
        _actions: &ActionsData,
        time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                (value * time.delta_secs()).into()
            }
            ActionValue::Axis1D(value) => (value * time.delta_secs()).into(),
            ActionValue::Axis2D(value) => (value * time.delta_secs()).into(),
            ActionValue::Axis3D(value) => (value * time.delta_secs()).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn scaling() {
        let actions = ActionsData::default();
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(500));

        assert_eq!(DeltaScale.apply(&actions, &time, true.into()), 0.5.into());
        assert_eq!(DeltaScale.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(DeltaScale.apply(&actions, &time, 0.5.into()), 0.25.into());
        assert_eq!(
            DeltaScale.apply(&actions, &time, Vec2::ONE.into()),
            (0.5, 0.5).into()
        );
        assert_eq!(
            DeltaScale.apply(&actions, &time, Vec3::ONE.into()),
            (0.5, 0.5, 0.5).into()
        );
    }
}
