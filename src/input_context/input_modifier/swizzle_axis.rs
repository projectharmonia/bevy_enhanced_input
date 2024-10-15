use bevy::prelude::*;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Swizzle axis components of an input value.
///
/// Useful to map a 1D input onto the Y axis of a 2D action.
///
/// Can't be applied to [`ActionValue::Bool`] and [`ActionValue::Axis1D`].
#[derive(Debug)]
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
