use bevy::prelude::*;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Response curve exponential.
///
/// Apply a simple exponential response curve to input values, per axis.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct ExponentialCurve {
    /// Curve exponent.
    pub exponent: Vec3,
}

impl ExponentialCurve {
    fn curve(value: f32, exponent: f32) -> f32 {
        if value != 1.0 {
            value.signum() * value.abs().powf(exponent)
        } else {
            value
        }
    }
}

impl Default for ExponentialCurve {
    fn default() -> Self {
        Self {
            exponent: Vec3::ONE,
        }
    }
}

impl InputModifier for ExponentialCurve {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                super::ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => Self::curve(value, self.exponent.x).into(),
            ActionValue::Axis2D(mut value) => {
                value.x = Self::curve(value.x, self.exponent.x);
                value.y = Self::curve(value.y, self.exponent.y);
                value.into()
            }
            ActionValue::Axis3D(mut value) => {
                value.x = Self::curve(value.x, self.exponent.x);
                value.y = Self::curve(value.y, self.exponent.y);
                value.y = Self::curve(value.z, self.exponent.z);
                value.into()
            }
        }
    }
}
