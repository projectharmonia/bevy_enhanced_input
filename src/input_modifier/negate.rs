use bevy::prelude::*;

use super::InputModifier;
use crate::{action_value::ActionValue, input_context::ActionsData};

/// Inverts value per axis.
///
/// By default, all axes are inverted.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Clone, Copy, Debug)]
pub struct Negate {
    /// Whether to inverse the X axis.
    pub x: bool,

    /// Whether to inverse the Y axis.
    pub y: bool,

    /// Whether to inverse the Z axis.
    pub z: bool,
}

impl Negate {
    /// Returns [`Self`] with inversion for all axes set to `invert`
    #[must_use]
    pub fn splat(invert: bool) -> Self {
        Self {
            x: invert,
            y: invert,
            z: invert,
        }
    }

    /// Returns [`Self`] with none of the axes inverted.
    pub fn none() -> Self {
        Self::splat(false)
    }

    /// Returns [`Self`] with all of the axes inverted.
    pub fn all() -> Self {
        Self::splat(true)
    }

    /// Returns [`Self`] with the X axis inverted.
    #[must_use]
    pub fn x() -> Self {
        Self {
            x: true,
            ..Self::none()
        }
    }

    /// Returns [`Self`] with the Y axis inverted.
    #[must_use]
    pub fn y() -> Self {
        Self {
            y: true,
            ..Self::none()
        }
    }

    /// Returns [`Self`] with the Z axis inverted.
    #[must_use]
    pub fn z() -> Self {
        Self {
            z: true,
            ..Self::none()
        }
    }
}

impl InputModifier for Negate {
    fn apply(
        &mut self,
        _actions: &ActionsData,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                self.apply(_actions, _time, value.into())
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
        let mut modifier = Negate::x();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), (-1.0).into());
        assert_eq!(modifier.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&actions, &time, 0.5.into()), (-0.5).into());
        assert_eq!(
            modifier.apply(&actions, &time, Vec2::ONE.into()),
            (-1.0, 1.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, Vec3::ONE.into()),
            (-1.0, 1.0, 1.0).into(),
        );
    }

    #[test]
    fn y() {
        let mut modifier = Negate::y();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), 1.0.into());
        assert_eq!(modifier.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&actions, &time, 0.5.into()), 0.5.into());
        assert_eq!(
            modifier.apply(&actions, &time, Vec2::ONE.into()),
            (1.0, -1.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, Vec3::ONE.into()),
            (1.0, -1.0, 1.0).into(),
        );
    }

    #[test]
    fn z() {
        let mut modifier = Negate::z();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), 1.0.into());
        assert_eq!(modifier.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&actions, &time, 0.5.into()), 0.5.into());
        assert_eq!(
            modifier.apply(&actions, &time, Vec2::ONE.into()),
            Vec2::ONE.into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, Vec3::ONE.into()),
            (1.0, 1.0, -1.0).into(),
        );
    }

    #[test]
    fn all() {
        let mut modifier = Negate::all();
        let actions = ActionsData::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), (-1.0).into());
        assert_eq!(modifier.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&actions, &time, 0.5.into()), (-0.5).into());
        assert_eq!(
            modifier.apply(&actions, &time, Vec2::ONE.into()),
            Vec2::NEG_ONE.into(),
        );
        assert_eq!(
            modifier.apply(&actions, &time, Vec3::ONE.into()),
            Vec3::NEG_ONE.into(),
        );
    }
}
