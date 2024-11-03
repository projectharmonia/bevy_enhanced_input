use bevy::prelude::*;

use super::InputModifier;
use crate::{action_value::ActionValue, input_context::input_action::ActionsData};

/// Normalizes input if possible or returns zero.
///
/// Does nothing for [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct Normalize;

impl InputModifier for Normalize {
    fn apply(
        &mut self,
        _actions: &ActionsData,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(_) => value,
            ActionValue::Axis1D(value) => {
                if value != 0.0 {
                    1.0.into()
                } else {
                    value.into()
                }
            }
            ActionValue::Axis2D(value) => value.normalize_or_zero().into(),
            ActionValue::Axis3D(value) => value.normalize_or_zero().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization() {
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(Normalize.apply(&actions, &time, true.into()), true.into());
        assert_eq!(Normalize.apply(&actions, &time, false.into()), false.into());
        assert_eq!(Normalize.apply(&actions, &time, 0.5.into()), 1.0.into());
        assert_eq!(Normalize.apply(&actions, &time, 0.0.into()), 0.0.into());
        assert_eq!(
            Normalize.apply(&actions, &time, Vec2::ONE.into()),
            Vec2::ONE.normalize_or_zero().into(),
        );
        assert_eq!(
            Normalize.apply(&actions, &time, Vec3::ONE.into()),
            Vec3::ONE.normalize_or_zero().into(),
        );
    }
}
