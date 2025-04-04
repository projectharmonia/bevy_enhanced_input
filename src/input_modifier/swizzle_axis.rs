use bevy::prelude::*;

use super::InputModifier;
use crate::{action_map::ActionMap, action_value::ActionValue};

/// Swizzle axis components of an input value.
///
/// Useful for things like mapping a 1D input onto the Y axis of a 2D action.
///
/// It tries to preserve the original dimension. However, if an axis from the original
/// is promoted to a higher dimension, the value's type changes. Missing axes will be replaced with zero.
///
/// For example, [`ActionValue::Bool`] will remain unchanged for [`Self::XZY`] (X in the first place).
/// But for variants like [`Self::YXZ`] (where X becomes the second component), it will be
/// converted into [`ActionValue::Axis2D`] with Y set to the value.
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
}

impl InputModifier for SwizzleAxis {
    fn apply(
        &mut self,
        _action_map: &ActionMap,
        _time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                self.apply(_action_map, _time, value.into())
            }
            ActionValue::Axis1D(value) => match self {
                SwizzleAxis::YXZ | SwizzleAxis::ZXY => (Vec2::Y * value).into(),
                SwizzleAxis::ZYX | SwizzleAxis::YZX => (Vec3::Z * value).into(),
                SwizzleAxis::XZY => value.into(),
            },
            ActionValue::Axis2D(value) => match self {
                SwizzleAxis::YXZ => value.yx().into(),
                SwizzleAxis::ZYX => (0.0, value.y).into(),
                SwizzleAxis::XZY => (value.x, 0.0).into(),
                SwizzleAxis::YZX => (value.y, 0.0).into(),
                SwizzleAxis::ZXY => (0.0, value.x).into(),
            },
            ActionValue::Axis3D(value) => match self {
                SwizzleAxis::YXZ => value.yxz().into(),
                SwizzleAxis::ZYX => value.zyx().into(),
                SwizzleAxis::XZY => value.xzy().into(),
                SwizzleAxis::YZX => value.yzx().into(),
                SwizzleAxis::ZXY => value.zxy().into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yxz() {
        let mut modifier = SwizzleAxis::YXZ;
        let actions = ActionMap::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), Vec2::Y.into());
        assert_eq!(
            modifier.apply(&actions, &time, false.into()),
            Vec2::ZERO.into()
        );
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), Vec2::Y.into());
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0).into()),
            (1.0, 0.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0, 2.0).into()),
            (1.0, 0.0, 2.0).into(),
        );
    }

    #[test]
    fn zyx() {
        let mut modifier = SwizzleAxis::ZYX;
        let actions = ActionMap::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), Vec3::Z.into());
        assert_eq!(
            modifier.apply(&actions, &time, false.into()),
            Vec3::ZERO.into()
        );
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), Vec3::Z.into());
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0).into()),
            (0.0, 1.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0, 2.0).into()),
            (2.0, 1.0, 0.0).into(),
        );
    }

    #[test]
    fn xzy() {
        let mut modifier = SwizzleAxis::XZY;
        let actions = ActionMap::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), 1.0.into());
        assert_eq!(modifier.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0).into()),
            (0.0, 0.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0, 2.0).into()),
            (0.0, 2.0, 1.0).into(),
        );
    }

    #[test]
    fn yzx() {
        let mut modifier = SwizzleAxis::YZX;
        let actions = ActionMap::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), Vec3::Z.into());
        assert_eq!(
            modifier.apply(&actions, &time, false.into()),
            Vec3::ZERO.into()
        );
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), Vec3::Z.into());
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0).into()),
            (1.0, 0.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0, 2.0).into()),
            (1.0, 2.0, 0.0).into(),
        );
    }

    #[test]
    fn zxy() {
        let mut modifier = SwizzleAxis::ZXY;
        let actions = ActionMap::default();
        let time = Time::default();

        assert_eq!(modifier.apply(&actions, &time, true.into()), Vec2::Y.into());
        assert_eq!(
            modifier.apply(&actions, &time, false.into()),
            Vec2::ZERO.into()
        );
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), Vec2::Y.into());
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0).into()),
            (0.0, 0.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, (0.0, 1.0, 2.0).into()),
            (2.0, 0.0, 1.0).into(),
        );
    }
}
