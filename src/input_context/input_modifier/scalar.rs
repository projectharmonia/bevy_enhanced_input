use bevy::prelude::*;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Scales input by a set factor per axis.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Clone, Copy, Debug)]
pub struct Scalar {
    /// The scalar that will be applied to the input value.
    ///
    /// For example, with the scalar set to `Vec3::new(2.0, 2.0, 2.0)`, each input axis will be multiplied by 2.0.
    ///
    /// Does nothing for boolean values.
    pub scalar: Vec3,
}

impl Scalar {
    /// Creates a new scalar with all axes set to `value`.
    #[must_use]
    pub fn splat(value: f32) -> Self {
        Self::new(Vec3::splat(value))
    }

    #[must_use]
    pub fn new(scalar: Vec3) -> Self {
        Self { scalar }
    }
}

impl InputModifier for Scalar {
    fn apply(&mut self, _time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                (value * self.scalar.x).into()
            }
            ActionValue::Axis1D(value) => (value * self.scalar.x).into(),
            ActionValue::Axis2D(value) => (value * self.scalar.xy()).into(),
            ActionValue::Axis3D(value) => (value * self.scalar).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaling() {
        let mut modifier = Scalar::splat(2.0);
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), 2.0.into());
        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 2.0.into());
        assert_eq!(modifier.apply(&time, Vec2::ONE.into()), (2.0, 2.0).into());
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            (2.0, 2.0, 2.0).into()
        );
    }
}
