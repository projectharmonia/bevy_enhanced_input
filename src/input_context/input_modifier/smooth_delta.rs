use bevy::prelude::*;
use interpolation::Ease;
pub use interpolation::EaseFunction;

use super::{ignore_incompatible, InputModifier};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input_context::context_instance::ActionContext,
};

/// Normalized smooth delta
///
/// Produces a smoothed normalized delta of the current(new) and last(old) input value.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct SmoothDelta {
    /// Defines how value will be smoothed.
    pub kind: SmoothKind,

    /// Multiplier for delta time.
    pub speed: f32,

    old_value: Vec3,

    value_delta: Vec3,
}

impl SmoothDelta {
    #[must_use]
    pub fn new(kind: impl Into<SmoothKind>, speed: f32) -> Self {
        Self {
            kind: kind.into(),
            speed,
            old_value: Default::default(),
            value_delta: Default::default(),
        }
    }
}

impl InputModifier for SmoothDelta {
    fn apply(&mut self, _ctx: &ActionContext, delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            ignore_incompatible!(value);
        }

        let value = value.as_axis3d();
        let target_value_delta = (value - self.old_value).normalize_or_zero();
        self.old_value = value;

        let alpha = (delta * self.speed).min(1.0);
        self.value_delta = match self.kind {
            SmoothKind::EaseFunction(ease_function) => {
                let ease_alpha = alpha.calc(ease_function);
                self.value_delta.lerp(target_value_delta, ease_alpha)
            }
            SmoothKind::Linear => self.value_delta.lerp(target_value_delta, alpha),
        };

        ActionValue::Axis3D(self.value_delta).convert(dim)
    }
}

/// Behavior options for [`SmoothDelta`].
///
/// Describes how eased value should be computed.
#[derive(Clone, Copy, Debug)]
pub enum SmoothKind {
    /// Follows [`EaseFunction`].
    EaseFunction(EaseFunction),
    /// Linear interpolation, with no function.
    Linear,
}

impl From<EaseFunction> for SmoothKind {
    fn from(value: EaseFunction) -> Self {
        Self::EaseFunction(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_context::input_action::ActionsData;

    #[test]
    fn linear() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut modifier = SmoothDelta::new(SmoothKind::Linear, 1.0);
        let delta = 0.1;
        assert_eq!(modifier.apply(&ctx, delta, true.into()), true.into());
        assert_eq!(modifier.apply(&ctx, delta, 0.5.into()), 0.1.into());
        assert_eq!(modifier.apply(&ctx, delta, 1.0.into()), 0.19.into());
    }

    #[test]
    fn ease_function() {
        let ctx = ActionContext {
            world: &World::new(),
            actions: &ActionsData::default(),
            entities: &[],
        };

        let mut modifier = SmoothDelta::new(EaseFunction::QuadraticIn, 1.0);
        let delta = 0.2;
        assert_eq!(modifier.apply(&ctx, delta, true.into()), true.into());
        assert_eq!(modifier.apply(&ctx, delta, 0.5.into()), 0.040000003.into());
        assert_eq!(modifier.apply(&ctx, delta, 1.0.into()), 0.0784.into());
    }
}
