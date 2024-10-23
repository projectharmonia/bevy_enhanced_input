use bevy::prelude::*;

use super::{ignore_incompatible, InputModifier};
use crate::action_value::ActionValue;

/// Swizzle axis components of an input value.
///
/// Useful to map a 1D input onto the Y axis of a 2D action.
///
/// Can't be applied to [`ActionValue::Bool`] and [`ActionValue::Axis1D`].
#[derive(Clone, Copy, Debug)]
pub enum SwizzleAxis {
    /// Swap X and Y axis. Useful for binding 1D inputs to the Y axis for 2D actions.
    YXZ,
    /// Swap X and Z axis.
    ZYX,
    /// Swap Y and Z axis.
    XZY,
    /// Reorder all axes, Y first.
    YZX,
    /// Reorder all axes, Z first.
    ZXY,
    /// Use X for all axes.
    XXX,
    /// Use Y for all axes.
    YYY,
    /// Use Z for all axes.
    ZZZ,
}

impl InputModifier for SwizzleAxis {
    fn apply(&mut self, _time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) | ActionValue::Axis1D(_) => {
                ignore_incompatible!(value);
            }
            ActionValue::Axis2D(value) => match self {
                SwizzleAxis::YXZ => value.yx().into(),
                SwizzleAxis::ZYX => (0.0, value.y).into(),
                SwizzleAxis::XZY => (value.x, 0.0).into(),
                SwizzleAxis::YZX => (value.y, 0.0).into(),
                SwizzleAxis::ZXY => (0.0, value.x).into(),
                SwizzleAxis::XXX => value.xx().into(),
                SwizzleAxis::YYY => value.yy().into(),
                SwizzleAxis::ZZZ => Vec2::ZERO.into(),
            },
            ActionValue::Axis3D(value) => match self {
                SwizzleAxis::YXZ => value.yxz().into(),
                SwizzleAxis::ZYX => value.zyx().into(),
                SwizzleAxis::XZY => value.xzy().into(),
                SwizzleAxis::YZX => value.yzx().into(),
                SwizzleAxis::ZXY => value.zxy().into(),
                SwizzleAxis::XXX => value.xxx().into(),
                SwizzleAxis::YYY => value.yyy().into(),
                SwizzleAxis::ZZZ => value.zzz().into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yxz() {
        let mut swizzle = SwizzleAxis::YXZ;
        let time = Time::default();

        assert_eq!(swizzle.apply(&time, true.into()), true.into());
        assert_eq!(swizzle.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(swizzle.apply(&time, (0.0, 1.0).into()), (1.0, 0.0).into());
        assert_eq!(
            swizzle.apply(&time, (0.0, 1.0, 2.0).into()),
            (1.0, 0.0, 2.0).into(),
        );
    }

    #[test]
    fn zyx() {
        let mut swizzle = SwizzleAxis::ZYX;
        let time = Time::default();

        assert_eq!(swizzle.apply(&time, true.into()), true.into());
        assert_eq!(swizzle.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(swizzle.apply(&time, (0.0, 1.0).into()), (0.0, 1.0).into());
        assert_eq!(
            swizzle.apply(&time, (0.0, 1.0, 2.0).into()),
            (2.0, 1.0, 0.0).into(),
        );
    }

    #[test]
    fn xzy() {
        let mut modifier = SwizzleAxis::XZY;
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, (0.0, 1.0).into()), (0.0, 0.0).into());
        assert_eq!(
            modifier.apply(&time, (0.0, 1.0, 2.0).into()),
            (0.0, 2.0, 1.0).into(),
        );
    }

    #[test]
    fn yzx() {
        let mut modifier = SwizzleAxis::YZX;
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, (0.0, 1.0).into()), (1.0, 0.0).into());
        assert_eq!(
            modifier.apply(&time, (0.0, 1.0, 2.0).into()),
            (1.0, 2.0, 0.0).into(),
        );
    }

    #[test]
    fn zxy() {
        let mut modifier = SwizzleAxis::ZXY;
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, (0.0, 1.0).into()), (0.0, 0.0).into());
        assert_eq!(
            modifier.apply(&time, (0.0, 1.0, 2.0).into()),
            (2.0, 0.0, 1.0).into(),
        );
    }

    #[test]
    fn xxx() {
        let mut modifier = SwizzleAxis::XXX;
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, (0.0, 1.0).into()), (0.0, 0.0).into());
        assert_eq!(
            modifier.apply(&time, (0.0, 1.0, 2.0).into()),
            (0.0, 0.0, 0.0).into(),
        );
    }

    #[test]
    fn yyy() {
        let mut modifier = SwizzleAxis::YYY;
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, (0.0, 1.0).into()), (1.0, 1.0).into());
        assert_eq!(
            modifier.apply(&time, (0.0, 1.0, 2.0).into()),
            (1.0, 1.0, 1.0).into(),
        );
    }

    #[test]
    fn zzz() {
        let mut modifier = SwizzleAxis::ZZZ;
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, (0.0, 1.0).into()), (0.0, 0.0).into());
        assert_eq!(
            modifier.apply(&time, (0.0, 1.0, 2.0).into()),
            (2.0, 2.0, 2.0).into(),
        );
    }
}
