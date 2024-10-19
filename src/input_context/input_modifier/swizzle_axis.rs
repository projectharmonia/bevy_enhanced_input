use bevy::prelude::*;

use super::InputModifier;
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
}

impl InputModifier for SwizzleAxis {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) | ActionValue::Axis1D(_) => {
                super::ignore_incompatible!(value);
            }
            ActionValue::Axis2D(value) => match self {
                SwizzleAxis::YXZ => value.yx().into(),
                SwizzleAxis::ZYX => Vec2::new(0.0, value.y).into(),
                SwizzleAxis::XZY => Vec2::new(value.x, 0.0).into(),
                SwizzleAxis::YZX => Vec2::new(value.y, 0.0).into(),
                SwizzleAxis::ZXY => Vec2::new(0.0, value.x).into(),
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
        let world = World::new();

        let mut swizzle = SwizzleAxis::YXZ;
        assert_eq!(swizzle.apply(&world, 0.0, true.into()), true.into());
        assert_eq!(swizzle.apply(&world, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            swizzle.apply(&world, 0.0, Vec2::new(0.0, 1.0).into()),
            Vec2::new(1.0, 0.0).into(),
        );
        assert_eq!(
            swizzle.apply(&world, 0.0, Vec3::new(0.0, 1.0, 2.0).into()),
            Vec3::new(1.0, 0.0, 2.0).into(),
        );
    }

    #[test]
    fn zyx() {
        let world = World::new();

        let mut swizzle = SwizzleAxis::ZYX;
        assert_eq!(swizzle.apply(&world, 0.0, true.into()), true.into());
        assert_eq!(swizzle.apply(&world, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            swizzle.apply(&world, 0.0, Vec2::new(0.0, 1.0).into()),
            Vec2::new(0.0, 1.0).into(),
        );
        assert_eq!(
            swizzle.apply(&world, 0.0, Vec3::new(0.0, 1.0, 2.0).into()),
            Vec3::new(2.0, 1.0, 0.0).into(),
        );
    }

    #[test]
    fn xzy() {
        let world = World::new();

        let mut modifier = SwizzleAxis::XZY;
        assert_eq!(modifier.apply(&world, 0.0, true.into()), true.into());
        assert_eq!(modifier.apply(&world, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&world, 0.0, Vec2::new(0.0, 1.0).into()),
            Vec2::new(0.0, 0.0).into(),
        );
        assert_eq!(
            modifier.apply(&world, 0.0, Vec3::new(0.0, 1.0, 2.0).into()),
            Vec3::new(0.0, 2.0, 1.0).into(),
        );
    }

    #[test]
    fn yzx() {
        let world = World::new();

        let mut modifier = SwizzleAxis::YZX;
        assert_eq!(modifier.apply(&world, 0.0, true.into()), true.into());
        assert_eq!(modifier.apply(&world, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&world, 0.0, Vec2::new(0.0, 1.0).into()),
            Vec2::new(1.0, 0.0).into(),
        );
        assert_eq!(
            modifier.apply(&world, 0.0, Vec3::new(0.0, 1.0, 2.0).into()),
            Vec3::new(1.0, 2.0, 0.0).into(),
        );
    }

    #[test]
    fn zxy() {
        let world = World::new();

        let mut modifier = SwizzleAxis::ZXY;
        assert_eq!(modifier.apply(&world, 0.0, true.into()), true.into());
        assert_eq!(modifier.apply(&world, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&world, 0.0, Vec2::new(0.0, 1.0).into()),
            Vec2::new(0.0, 0.0).into(),
        );
        assert_eq!(
            modifier.apply(&world, 0.0, Vec3::new(0.0, 1.0, 2.0).into()),
            Vec3::new(2.0, 0.0, 1.0).into(),
        );
    }
}
