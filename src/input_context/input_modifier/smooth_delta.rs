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
    fn apply(&mut self, time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        if let ActionValue::Bool(value) = value {
            let value = if value { 1.0 } else { 0.0 };
            return self.apply(time, value.into());
        }

        let dim = value.dim();
        let value = value.as_axis3d();
        let target_value_delta = (value - self.old_value).normalize_or_zero();
        self.old_value = value;

        let alpha = (time.delta_seconds() * self.speed).min(1.0);
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
    use std::time::Duration;

    use super::*;

    #[test]
    fn linear_bool() {
        let mut modifier = SmoothDelta::new(SmoothKind::Linear, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, true.into()), 0.1.into());
    }

    #[test]
    fn ease_function_bool() {
        let mut modifier = SmoothDelta::new(EaseFunction::QuadraticIn, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(200));

        assert_eq!(modifier.apply(&time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, true.into()), 0.040000003.into());
    }

    #[test]
    fn linear_axis1d() {
        let mut modifier = SmoothDelta::new(SmoothKind::Linear, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(modifier.apply(&time, 0.5.into()), 0.1.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 0.19.into());
    }

    #[test]
    fn ease_function_axis1d() {
        let mut modifier = SmoothDelta::new(EaseFunction::QuadraticIn, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(200));

        assert_eq!(modifier.apply(&time, 0.5.into()), 0.040000003.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 0.0784.into());
    }

    #[test]
    fn linear_axis2d() {
        let mut modifier = SmoothDelta::new(SmoothKind::Linear, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(
            modifier.apply(&time, Vec2::splat(0.5).into()),
            Vec2::splat(0.07071068).into()
        );
        assert_eq!(
            modifier.apply(&time, Vec2::ONE.into()),
            Vec2::splat(0.1343503).into()
        );
    }

    #[test]
    fn ease_function_axis2d() {
        let mut modifier = SmoothDelta::new(EaseFunction::QuadraticIn, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(200));

        assert_eq!(
            modifier.apply(&time, Vec2::splat(0.5).into()),
            Vec2::splat(0.028284272).into()
        );
        assert_eq!(
            modifier.apply(&time, Vec2::ONE.into()),
            Vec2::splat(0.055437177).into()
        );
    }

    #[test]
    fn linear_axis3d() {
        let mut modifier = SmoothDelta::new(SmoothKind::Linear, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(100));

        assert_eq!(
            modifier.apply(&time, Vec3::splat(0.5).into()),
            Vec3::splat(0.057735026).into()
        );
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            Vec3::splat(0.10969655).into()
        );
    }

    #[test]
    fn ease_function_axis3d() {
        let mut modifier = SmoothDelta::new(EaseFunction::QuadraticIn, 1.0);
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(200));

        assert_eq!(
            modifier.apply(&time, Vec3::splat(0.5).into()),
            Vec3::splat(0.023094011).into()
        );
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            Vec3::splat(0.045264263).into()
        );
    }
}
