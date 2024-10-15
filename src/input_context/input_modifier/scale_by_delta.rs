use bevy::prelude::*;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Multiplies the input value by delta time for this frame.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct ScaleByDelta;

impl InputModifier for ScaleByDelta {
    fn apply(&mut self, _world: &World, delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                super::ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => (value * delta).into(),
            ActionValue::Axis2D(value) => (value * delta).into(),
            ActionValue::Axis3D(value) => (value * delta).into(),
        }
    }
}
