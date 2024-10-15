use bevy::prelude::*;

use super::InputModifier;
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

    /// Threshold below which input is ignored.
    pub upper_threshold: f32,
}

impl DeadZone {
    fn dead_zone(self, axis_value: f32) -> f32 {
        // Translate and scale the input to the +/- 1 range after removing the dead zone.
        let lower_bound = (axis_value.abs() - self.lower_threshold).max(0.0);
        let scaled_value = lower_bound / (self.upper_threshold - self.lower_threshold);
        scaled_value.min(1.0) * axis_value.signum()
    }
}

impl Default for DeadZone {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            lower_threshold: 0.2,
            upper_threshold: 1.0,
        }
    }
}

impl InputModifier for DeadZone {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                super::ignore_incompatible!(value);
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
    // Apply dead zone logic to all axes simultaneously.
    //
    // This gives smooth input (circular/spherical coverage). On a 1d axis input this works identically to [`Self::Axial`].
    #[default]
    Radial,

    // Apply dead zone to axes individually.
    //
    // This will result in input being chamfered at the corners for 2d/3d axis inputs.
    Axial,
}
