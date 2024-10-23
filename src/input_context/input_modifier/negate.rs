use bevy::prelude::*;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Inverts value per axis.
///
/// By default, all axes are inverted.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
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
    fn apply(&mut self, _time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                self.apply(_time, value.into())
            }
            ActionValue::Axis1D(value) => {
                if self.x {
                    (-value).into()
                } else {
                    value.into()
                }
            }
            ActionValue::Axis2D(mut value) => {
                if self.x {
                    value.x = -value.x;
                }
                if self.y {
                    value.y = -value.y;
                }
                value.into()
            }
            ActionValue::Axis3D(mut value) => {
                if self.x {
                    value.x = -value.x;
                }
                if self.y {
                    value.y = -value.y;
                }
                if self.z {
                    value.z = -value.z;
                }
                value.into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x() {
        let mut modifier = Negate::x(true);
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), (-1.0).into());
        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, 0.5.into()), (-0.5).into());
        assert_eq!(modifier.apply(&time, Vec2::ONE.into()), (-1.0, 1.0).into());
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            (-1.0, 1.0, 1.0).into(),
        );
    }

    #[test]
    fn y() {
        let mut modifier = Negate::y(true);
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, 0.5.into()), 0.5.into());
        assert_eq!(modifier.apply(&time, Vec2::ONE.into()), (1.0, -1.0).into());
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            (1.0, -1.0, 1.0).into(),
        );
    }

    #[test]
    fn z() {
        let mut modifier = Negate::z(true);
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, 0.5.into()), 0.5.into());
        assert_eq!(modifier.apply(&time, Vec2::ONE.into()), Vec2::ONE.into());
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            (1.0, 1.0, -1.0).into(),
        );
    }

    #[test]
    fn all() {
        let mut modifier = Negate::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), (-1.0).into());
        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, 0.5.into()), (-0.5).into());
        assert_eq!(
            modifier.apply(&time, Vec2::ONE.into()),
            Vec2::NEG_ONE.into(),
        );
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            Vec3::NEG_ONE.into(),
        );
    }
}
