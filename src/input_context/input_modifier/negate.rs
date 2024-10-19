use bevy::prelude::*;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Inverts value per axis.
///
/// By default, all axes are inverted.
#[derive(Clone, Copy, Debug)]
pub struct Negate {
    /// Wheter to inverse the X axis.
    pub x: bool,

    /// Wheter to inverse the Y axis.
    pub y: bool,

    /// Wheter to inverse the Z axis.
    pub z: bool,
}

impl Negate {
    /// Returns [`Self`] with invertion for all axes set to `invert`
    #[must_use]
    pub fn all(invert: bool) -> Self {
        Self {
            x: invert,
            y: invert,
            z: invert,
        }
    }

    /// Returns [`Self`] with invertion for x set to `invert`
    #[must_use]
    pub fn x(invert: bool) -> Self {
        Self {
            x: invert,
            y: false,
            z: false,
        }
    }

    /// Returns [`Self`] with invertion for y set to `invert`
    #[must_use]
    pub fn y(invert: bool) -> Self {
        Self {
            x: false,
            y: invert,
            z: false,
        }
    }

    /// Returns [`Self`] with invertion for z set to `invert`
    #[must_use]
    pub fn z(invert: bool) -> Self {
        Self {
            x: false,
            y: false,
            z: invert,
        }
    }
}

impl Default for Negate {
    fn default() -> Self {
        Self::all(true)
    }
}

impl InputModifier for Negate {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        let x = if self.x { -1.0 } else { 1.0 };
        let y = if self.y { -1.0 } else { 1.0 };
        let z = if self.z { -1.0 } else { 1.0 };
        let negated = value.as_axis3d() * Vec3::new(x, y, z);

        ActionValue::Axis3D(negated).convert(value.dim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negation() {
        let world = World::new();

        assert_eq!(
            Negate::default().apply(&world, 0.0, Vec3::ONE.into()),
            Vec3::NEG_ONE.into(),
        );
        assert_eq!(
            Negate::x(true).apply(&world, 0.0, Vec3::ONE.into()),
            Vec3::new(-1.0, 1.0, 1.0).into(),
        );
        assert_eq!(
            Negate::y(true).apply(&world, 0.0, Vec3::ONE.into()),
            Vec3::new(1.0, -1.0, 1.0).into(),
        );
        assert_eq!(
            Negate::z(true).apply(&world, 0.0, Vec3::ONE.into()),
            Vec3::new(1.0, 1.0, -1.0).into(),
        );
    }
}
