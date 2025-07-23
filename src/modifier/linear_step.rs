use bevy::prelude::*;

use crate::prelude::*;

/// Gradually steps the input value toward the current value at a constant linear rate.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`]
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct LinearStep {
    /// The fraction of the distance to step per frame.
    ///
    /// Must be between `0.0` and `1.0`, where `0.0` results
    /// in no movement and `1.0` snaps directly to the current value.
    pub step_rate: f32,
    previous_value: Vec3,
}

impl LinearStep {
    #[must_use]
    pub const fn new(step_rate: f32) -> Self {
        Self {
            step_rate,
            previous_value: Vec3::ZERO,
        }
    }
}

impl InputModifier for LinearStep {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        if !(0.0..=1.0).contains(&self.step_rate) {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!("step rate can't be outside 0.0..=1.0: {}", self.step_rate);
            return value;
        }

        let target_value = value.as_axis3d();
        if (0.0..self.step_rate).contains(&self.previous_value.distance_squared(target_value)) {
            self.previous_value = target_value;
            return value;
        }
        let difference = target_value.length() - self.previous_value.length();
        if difference == 0.0 {
            return value;
        }
        if difference > 0.0 {
            self.previous_value += self.step_rate * target_value;
        } else {
            self.previous_value -= self.step_rate * self.previous_value.signum();
        }

        ActionValue::Axis3D(self.previous_value).convert(value.dim())
    }
}
