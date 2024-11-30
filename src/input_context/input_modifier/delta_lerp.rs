use bevy::prelude::*;

use super::InputModifier;
use crate::{action_value::ActionValue, input_context::context_instance::ActionsData};

/// Produces a smoothed value of the current and previous input value.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Clone, Copy, Debug)]
pub struct DeltaLerp {
    /// Multiplier for delta time, determines the rate of smoothing.
    ///
    /// By default set to 8.0, an ad-hoc value that usually produces nice results.
    pub speed: f32,

    prev_value: Vec3,
}

impl DeltaLerp {
    #[must_use]
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            prev_value: Default::default(),
        }
    }
}

impl Default for DeltaLerp {
    fn default() -> Self {
        Self::new(8.0)
    }
}

impl InputModifier for DeltaLerp {
    fn apply(
        &mut self,
        _actions: &ActionsData,
        time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        if let ActionValue::Bool(value) = value {
            let value = if value { 1.0 } else { 0.0 };
            return self.apply(_actions, time, value.into());
        }

        let target_value = value.as_axis3d();
        if self.prev_value.distance_squared(target_value) < 1e-4 {
            // Snap to the target if the distance is too small.
            self.prev_value = target_value;
            return value;
        }

        let alpha = time.delta_seconds() * self.speed;
        let smoothed = self.prev_value.lerp(target_value, alpha);
        self.prev_value = smoothed;

        ActionValue::Axis3D(smoothed).convert(value.dim())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn lerp() {
        let mut modifier = DeltaLerp::new(1.0); // Use 1.0 for simpler calculations.
        let actions = ActionsData::default();
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(modifier.apply(&actions, &time, 0.5.into()), 0.05.into());
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 0.145.into());
    }

    #[test]
    fn bool_as_axis1d() {
        let mut modifier = DeltaLerp::new(1.0);
        let actions = ActionsData::default();
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(modifier.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&actions, &time, true.into()), 0.1.into());
    }

    #[test]
    fn snapping() {
        let mut modifier = DeltaLerp::default();
        let actions = ActionsData::default();
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        modifier.prev_value = Vec3::X * 0.99;
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());

        modifier.prev_value = Vec3::X * 0.98;
        assert_ne!(modifier.apply(&actions, &time, 1.0.into()), 1.0.into());
    }
}
