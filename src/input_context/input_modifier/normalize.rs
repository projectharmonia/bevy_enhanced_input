use bevy::prelude::*;

use super::{ignore_incompatible, InputModifier};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input_context::context_instance::ActionContext,
};

/// Normalizes input if possible or returns zero.
#[derive(Clone, Copy, Debug)]
pub struct Normalize;

impl InputModifier for Normalize {
    fn apply(&mut self, _ctx: &ActionContext, _delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool || dim == ActionValueDim::Axis1D {
            ignore_incompatible!(value);
        }

        let normalized = value.as_axis3d().normalize_or_zero();
        ActionValue::Axis3D(normalized).convert(dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn normalization() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        assert_eq!(Normalize.apply(&ctx, 0.0, true.into()), true.into());
        assert_eq!(Normalize.apply(&ctx, 0.0, 0.5.into()), 0.5.into());
        assert_eq!(
            Normalize.apply(&ctx, 0.0, Vec2::ZERO.into()),
            Vec2::ZERO.into(),
        );
        assert_eq!(
            Normalize.apply(&ctx, 0.0, Vec2::ONE.into()),
            Vec2::ONE.normalize_or_zero().into(),
        );
        assert_eq!(
            Normalize.apply(&ctx, 0.0, Vec3::ONE.into()),
            Vec3::ONE.normalize_or_zero().into(),
        );
    }
}
