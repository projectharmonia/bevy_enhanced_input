use bevy::prelude::*;

use super::{ignore_incompatible, InputModifier};
use crate::action_value::ActionValue;

/// Input values within the range [Self::lower_threshold] -> [Self::upper_threshold] will be remapped from 0 -> 1.
/// Values outside this range will be clamped.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct DeadZone {
    pub kind: DeadZoneKind,

    /// Threshold below which input is ignored.
    pub lower_threshold: f32,

    /// Threshold above which input is clamped to 1.
    pub upper_threshold: f32,
}

impl DeadZone {
    #[must_use]
    pub fn new(kind: DeadZoneKind) -> Self {
        Self {
            kind,
            lower_threshold: 0.2,
            upper_threshold: 1.0,
        }
    }

    #[must_use]
    pub fn with_lower_threshold(mut self, lower_threshold: f32) -> Self {
        self.lower_threshold = lower_threshold;
        self
    }

    #[must_use]
    pub fn with_upper_threshold(mut self, upper_threshold: f32) -> Self {
        self.upper_threshold = upper_threshold;
        self
    }

    fn dead_zone(self, axis_value: f32) -> f32 {
        // Translate and scale the input to the +/- 1 range after removing the dead zone.
        let lower_bound = (axis_value.abs() - self.lower_threshold).max(0.0);
        let scaled_value = lower_bound / (self.upper_threshold - self.lower_threshold);
        scaled_value.min(1.0) * axis_value.signum()
    }
}

impl Default for DeadZone {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl InputModifier for DeadZone {
    fn apply(&mut self, _time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => self.dead_zone(value).into(),
            ActionValue::Axis2D(mut value) => match self.kind {
                DeadZoneKind::Radial => {
                    (value.normalize_or_zero() * self.dead_zone(value.length())).into()
                }
                DeadZoneKind::Axial => {
                    value.x = self.dead_zone(value.x);
                    value.y = self.dead_zone(value.y);
                    value.into()
                }
            },
            ActionValue::Axis3D(mut value) => match self.kind {
                DeadZoneKind::Radial => {
                    (value.normalize_or_zero() * self.dead_zone(value.length())).into()
                }
                DeadZoneKind::Axial => {
                    value.x = self.dead_zone(value.x);
                    value.y = self.dead_zone(value.y);
                    value.z = self.dead_zone(value.z);
                    value.into()
                }
            },
        }
    }
}

/// Dead zone behavior.
#[derive(Default, Clone, Copy, Debug)]
pub enum DeadZoneKind {
    /// Apply dead zone logic to all axes simultaneously.
    ///
    /// This gives smooth input (circular/spherical coverage).
    /// For [`ActionValue::Axis1D`] this works identically to [`Self::Axial`].
    #[default]
    Radial,
    /// Apply dead zone to axes individually.
    ///
    /// This will result in input being chamfered at the corners
    /// for [`ActionValue::Axis2D`]/[`ActionValue::Axis2D`].
    Axial,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radial() {
        let mut modifier = DeadZone::new(DeadZoneKind::Radial);
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, 0.5.into()), 0.375.into());
        assert_eq!(modifier.apply(&time, 0.2.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, 2.0.into()), 1.0.into());

        assert_eq!(
            modifier.apply(&time, (Vec2::ONE * 0.5).into()),
            (Vec2::ONE * 0.4482233).into()
        );
        assert_eq!(
            modifier.apply(&time, Vec2::ONE.into()),
            (Vec2::ONE * 0.70710677).into()
        );
        assert_eq!(
            modifier.apply(&time, (Vec2::ONE * 0.2).into()),
            (Vec2::ONE * 0.07322331).into()
        );

        assert_eq!(
            modifier.apply(&time, (Vec3::ONE * 0.5).into()),
            (Vec3::ONE * 0.48066244).into()
        );
        assert_eq!(
            modifier.apply(&time, Vec3::ONE.into()),
            (Vec3::ONE * 0.57735026).into()
        );
        assert_eq!(
            modifier.apply(&time, (Vec3::ONE * 0.2).into()),
            (Vec3::ONE * 0.105662435).into()
        );
    }

    #[test]
    fn axial() {
        let mut modifier = DeadZone::new(DeadZoneKind::Axial);
        let time = Time::default();

        assert_eq!(modifier.apply(&time, true.into()), true.into());
        assert_eq!(modifier.apply(&time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.apply(&time, 0.5.into()), 0.375.into());
        assert_eq!(modifier.apply(&time, 0.2.into()), 0.0.into());
        assert_eq!(modifier.apply(&time, 2.0.into()), 1.0.into());

        assert_eq!(
            modifier.apply(&time, (Vec2::ONE * 0.5).into()),
            (Vec2::ONE * 0.375).into()
        );
        assert_eq!(modifier.apply(&time, Vec2::ONE.into()), Vec2::ONE.into());
        assert_eq!(
            modifier.apply(&time, (Vec2::ONE * 0.2).into()),
            Vec2::ZERO.into()
        );

        assert_eq!(
            modifier.apply(&time, (Vec3::ONE * 0.5).into()),
            (Vec3::ONE * 0.375).into()
        );
        assert_eq!(modifier.apply(&time, Vec3::ONE.into()), Vec3::ONE.into());
        assert_eq!(
            modifier.apply(&time, (Vec3::ONE * 0.2).into()),
            Vec3::ZERO.into()
        );
    }
}
