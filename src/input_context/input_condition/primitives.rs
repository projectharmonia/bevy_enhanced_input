use bevy::prelude::*;

use crate::action_value::ActionValue;

/// Helper for building triggers that have firing conditions governed by elapsed time.
#[derive(Default, Debug)]
pub struct HeldTimer {
    /// If set to `true`, [`Time::relative_speed`] will be applied to the held duration.
    ///
    /// By default is set to `false`.
    pub relative_to_speed: bool,

    duration: f32,
}

impl HeldTimer {
    pub fn relative_to_speed(relative_to_speed: bool) -> Self {
        Self {
            relative_to_speed,
            duration: 0.0,
        }
    }

    pub fn update(&mut self, world: &World, mut delta: f32) {
        if self.relative_to_speed {
            let time = world.resource::<Time<Virtual>>();
            delta *= time.relative_speed()
        }

        self.duration += delta;
    }

    pub fn reset(&mut self) {
        self.duration = 0.0;
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }
}

/// Value at which a button considered actuated.
#[derive(Clone, Copy, Debug)]
pub struct Actuation(pub f32);

impl Actuation {
    /// Returns `true` if the value in sufficiently large.
    pub fn is_actuated(self, value: ActionValue) -> bool {
        let value = match value {
            ActionValue::Bool(value) => {
                if value {
                    1.0
                } else {
                    0.0
                }
            }
            ActionValue::Axis1D(value) => value * value,
            ActionValue::Axis2D(value) => value.length_squared(),
            ActionValue::Axis3D(value) => value.length_squared(),
        };

        value >= self.0 * self.0
    }
}

impl Default for Actuation {
    fn default() -> Self {
        Self(0.5)
    }
}

impl From<f32> for Actuation {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
