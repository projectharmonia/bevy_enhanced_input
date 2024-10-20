use bevy::prelude::*;

use super::{ignore_incompatible, InputModifier};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input_context::context_instance::ActionContext,
};

/// Multiplies the input value by delta time for this frame.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct ScaleByDelta;

impl InputModifier for ScaleByDelta {
    fn apply(&mut self, _ctx: &ActionContext, delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            ignore_incompatible!(value);
        }

        ActionValue::Axis3D(value.as_axis3d() * delta).convert(dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn scaling() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let delta = 0.5;
        assert_eq!(ScaleByDelta.apply(&ctx, delta, true.into()), true.into());
        assert_eq!(ScaleByDelta.apply(&ctx, delta, 0.5.into()), 0.25.into());
        assert_eq!(
            ScaleByDelta.apply(&ctx, delta, Vec2::ONE.into()),
            (0.5, 0.5).into()
        );
        assert_eq!(
            ScaleByDelta.apply(&ctx, delta, Vec3::ONE.into()),
            (0.5, 0.5, 0.5).into()
        );
    }
}
