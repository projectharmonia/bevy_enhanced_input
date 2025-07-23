use bevy::prelude::*;

use crate::prelude::*;

/// Linearly accelerates the input value by `step_rate` each frame.
///
/// Value must be between `0.0` and `1.0`
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`]
#[derive(Clone, Copy, Debug)]
pub struct LinearAccelerate {
    pub step_rate: f32,
    current_value: Vec3,
}

impl LinearAccelerate {
    #[must_use]
    pub fn new(step_rate: f32) -> Self {
        Self {
            step_rate,
            current_value: Default::default(),
        }
    }
}

impl InputModifier for LinearAccelerate {
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
        if (0.0..self.step_rate).contains(&self.current_value.distance_squared(target_value)) {
            self.current_value = target_value;
            return value;
        }
        let difference = target_value.length() - self.current_value.length();
        if difference == 0.0 {
            return value;
        }
        if difference > 0.0 {
            self.current_value += self.step_rate * target_value;
        } else {
            self.current_value -= self.step_rate * self.current_value.signum();
        }

        ActionValue::Axis3D(self.current_value).convert(value.dim())
    }
}
