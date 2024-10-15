use bevy::prelude::*;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Scales input by a set factor per axis.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct Scalar {
    /// The scalar that will be applied to the input value.
    ///
    /// For example, with the scalar set to `Vec3::new(2.0, 2.0, 2.0)`, each input axis will be multiplied by 2.0.
    ///
    /// Does nothing for boolean values.
    pub scalar: Vec3,
}

impl InputModifier for Scalar {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                super::ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => (value * self.scalar.x).into(),
            ActionValue::Axis2D(value) => (value * self.scalar.xy()).into(),
            ActionValue::Axis3D(value) => (value * self.scalar).into(),
        }
    }
}
