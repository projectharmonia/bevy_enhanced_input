use bevy::prelude::*;

use super::InputModifier;
use crate::{action_value::ActionValue, ActionValueDim};

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

impl Scalar {
    pub fn new(scalar: Vec3) -> Self {
        Self { scalar }
    }
}

impl InputModifier for Scalar {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            super::ignore_incompatible!(value);
        }

        ActionValue::Axis3D(value.as_axis3d() * self.scalar).convert(dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaling() {
        let world = World::new();

        let mut scalar = Scalar::new(Vec3::ONE * 2.0);
        assert_eq!(scalar.apply(&world, 0.0, true.into()), true.into());
        assert_eq!(scalar.apply(&world, 0.0, 1.0.into()), 2.0.into());
        assert_eq!(
            scalar.apply(&world, 0.0, Vec2::ONE.into()),
            Vec2::new(2.0, 2.0).into()
        );
        assert_eq!(
            scalar.apply(&world, 0.0, Vec3::ONE.into()),
            Vec3::new(2.0, 2.0, 2.0).into()
        );
    }
}
