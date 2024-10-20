use bevy::prelude::*;

use super::InputModifier;
use crate::{action_value::ActionValue, input_context::context_instance::ActionContext};

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
    fn apply(&mut self, _ctx: &ActionContext, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) | ActionValue::Axis1D(_) => {
                super::ignore_incompatible!(value);
            }
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
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn yxz() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut swizzle = SwizzleAxis::YXZ;
        assert_eq!(swizzle.apply(&ctx, 0.0, true.into()), true.into());
        assert_eq!(swizzle.apply(&ctx, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            swizzle.apply(&ctx, 0.0, (0.0, 1.0).into()),
            (1.0, 0.0).into(),
        );
        assert_eq!(
            swizzle.apply(&ctx, 0.0, (0.0, 1.0, 2.0).into()),
            (1.0, 0.0, 2.0).into(),
        );
    }

    #[test]
    fn zyx() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut swizzle = SwizzleAxis::ZYX;
        assert_eq!(swizzle.apply(&ctx, 0.0, true.into()), true.into());
        assert_eq!(swizzle.apply(&ctx, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            swizzle.apply(&ctx, 0.0, (0.0, 1.0).into()),
            (0.0, 1.0).into(),
        );
        assert_eq!(
            swizzle.apply(&ctx, 0.0, (0.0, 1.0, 2.0).into()),
            (2.0, 1.0, 0.0).into(),
        );
    }

    #[test]
    fn xzy() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut modifier = SwizzleAxis::XZY;
        assert_eq!(modifier.apply(&ctx, 0.0, true.into()), true.into());
        assert_eq!(modifier.apply(&ctx, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&ctx, 0.0, (0.0, 1.0).into()),
            (0.0, 0.0).into(),
        );
        assert_eq!(
            modifier.apply(&ctx, 0.0, (0.0, 1.0, 2.0).into()),
            (0.0, 2.0, 1.0).into(),
        );
    }

    #[test]
    fn yzx() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut modifier = SwizzleAxis::YZX;
        assert_eq!(modifier.apply(&ctx, 0.0, true.into()), true.into());
        assert_eq!(modifier.apply(&ctx, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&ctx, 0.0, (0.0, 1.0).into()),
            (1.0, 0.0).into(),
        );
        assert_eq!(
            modifier.apply(&ctx, 0.0, (0.0, 1.0, 2.0).into()),
            (1.0, 2.0, 0.0).into(),
        );
    }

    #[test]
    fn zxy() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut modifier = SwizzleAxis::ZXY;
        assert_eq!(modifier.apply(&ctx, 0.0, true.into()), true.into());
        assert_eq!(modifier.apply(&ctx, 0.0, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&ctx, 0.0, (0.0, 1.0).into()),
            (0.0, 0.0).into(),
        );
        assert_eq!(
            modifier.apply(&ctx, 0.0, (0.0, 1.0, 2.0).into()),
            (2.0, 0.0, 1.0).into(),
        );
    }
}
