use bevy::prelude::*;
use interpolation::Ease;
pub use interpolation::EaseFunction;

use super::InputModifier;
use crate::action_value::{ActionValue, ActionValueDim};

/// Normalized smooth delta
///
/// Produces a smoothed normalized delta of the current(new) and last(old) input value.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Debug)]
pub struct SmoothDelta {
    /// Defines how value will be smoothed.
    pub smoothing_method: SmoothingMethod,

    /// Speed or alpha.
    ///
    /// If the speed given is 0, then jump to the target.
    pub speed: f32,

    old_value: Vec3,

    value_delta: Vec3,
}

impl SmoothDelta {
    #[must_use]
    pub fn new(smoothing_method: SmoothingMethod, speed: f32) -> Self {
        Self {
            smoothing_method,
            speed,
            old_value: Default::default(),
            value_delta: Default::default(),
        }
    }
}

impl InputModifier for SmoothDelta {
    fn apply(&mut self, _world: &World, delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            super::ignore_incompatible!(value);
        }

        let value = value.as_axis3d();
        let target_value_delta = (self.old_value - value).normalize_or_zero();
        self.old_value = value;

        let normalized_delta = delta / self.speed;
        self.value_delta = match self.smoothing_method {
            SmoothingMethod::EaseFunction(ease_function) => {
                let ease_delta = normalized_delta.calc(ease_function);
                self.value_delta.lerp(target_value_delta, ease_delta)
            }
            SmoothingMethod::Linear => self.value_delta.lerp(target_value_delta, normalized_delta),
        };

        ActionValue::Axis3D(self.value_delta).convert(dim)
    }
}

/// Behavior options for [`SmoothDelta`].
///
/// Describe how eased value should be computed.
#[derive(Clone, Copy, Debug)]
pub enum SmoothingMethod {
    /// Follow [`EaseFunction`].
    EaseFunction(EaseFunction),
    /// Linear interpolation, with no function.
    Linear,
}
