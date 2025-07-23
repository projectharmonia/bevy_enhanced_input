use bevy::prelude::*;

use crate::prelude::*;

/// Gradually steps the input value toward the target value at a constant linear rate.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`]
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct LinearStep {
    /// The fraction of the distance to step per frame.
    ///
    /// Must be between `0.0` and `1.0`, where `0.0` results
    /// in no movement and `1.0` snaps directly to the target value.
    pub step_rate: f32,

    current_value: Vec3,
}

impl LinearStep {
    #[must_use]
    pub const fn new(step_rate: f32) -> Self {
        Self {
            step_rate,
            current_value: Vec3::ZERO,
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

        if let ActionValue::Bool(value) = value {
            let value = if value { 1.0 } else { 0.0 };
            return self.transform(_actions, _time, value.into());
        }

        let target_value = value.as_axis3d();

        // Snap if distance is less than one step.
        let distance = self.current_value.distance(target_value);
        if distance <= self.step_rate {
            self.current_value = target_value;
            return value;
        }

        let diff = target_value - self.current_value;
        self.current_value += diff * self.step_rate;

        ActionValue::Axis3D(self.current_value).convert(value.dim())
    }
}
