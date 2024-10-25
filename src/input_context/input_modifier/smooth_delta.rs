use bevy::prelude::*;
use interpolation::Ease;
pub use interpolation::EaseFunction;

use super::InputModifier;
use crate::action_value::ActionValue;

/// Normalized smooth delta
///
/// Produces a smoothed normalized delta of the current(new) and last(old) input value.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Clone, Copy, Debug)]
pub struct SmoothDelta {
    /// Defines how value will be smoothed.
    pub kind: SmoothKind,

    /// Multiplier for delta time, determines the rate of smoothing.
    ///
    /// By default set to 8.0, an ad-hoc value that usually produces nice results.
    pub speed: f32,

    prev_value: Vec3,
}

impl SmoothDelta {
    #[must_use]
    pub fn new(kind: impl Into<SmoothKind>) -> Self {
        Self {
            kind: kind.into(),
            speed: 8.0,
            prev_value: Default::default(),
        }
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

impl InputModifier for SmoothDelta {
    fn apply(&mut self, time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        if let ActionValue::Bool(value) = value {
            let value = if value { 1.0 } else { 0.0 };
            return self.apply(time, value.into());
        }

        let target_value = value.as_axis3d();
        if self.prev_value.distance_squared(target_value) < 1e-4 {
            // Snap to the target if the distance is too small.
            self.prev_value = target_value;
            return value;
        }

        let alpha = time.delta_seconds() * self.speed;
        let smoothed = match self.kind {
            SmoothKind::EaseFunction(ease_function) => {
                let ease_alpha = alpha.calc(ease_function);
                self.prev_value.lerp(target_value, ease_alpha)
            }
            SmoothKind::Linear => self.prev_value.lerp(target_value, alpha),
        };
        self.prev_value = smoothed;

        ActionValue::Axis3D(smoothed).convert(value.dim())
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
    use std::time::Duration;

    use super::*;

    #[test]
    fn linear() {
        let mut modifier = SmoothDelta::new(SmoothKind::Linear).with_speed(1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(modifier.apply(&time, 0.5.into()), 0.05.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 0.145.into());
    }

    #[test]
    fn ease_function() {
        let mut modifier = SmoothDelta::new(EaseFunction::QuadraticIn).with_speed(1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(200));

        assert_eq!(modifier.apply(&time, 0.5.into()), 0.020000001.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 0.059200004.into());
    }

    #[test]
    fn bool_as_axis1d() {
        let mut modifier = SmoothDelta::new(SmoothKind::Linear).with_speed(1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, true.into()), 0.1.into());
    }

    #[test]
    fn snapping() {
        let mut modifier = SmoothDelta::new(SmoothKind::Linear).with_speed(1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        modifier.prev_value = Vec3::X * 0.99;
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());

        modifier.prev_value = Vec3::X * 0.98;
        assert_ne!(modifier.apply(&time, 1.0.into()), 1.0.into());
    }
}
