use bevy::prelude::*;

use super::InputModifier;
use crate::{action_value::ActionValue, ActionValueDim};

/// Smooth inputs out over multiple frames.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Debug)]
pub struct Smooth {
    /// How long input has been zero.
    zero_time: f32,

    /// Current average input/sample.
    average_value: Vec3,

    /// Number of samples since input has been zero.
    samples: u32,

    /// Input sampling total time.
    total_sample_time: f32,
}

impl InputModifier for Smooth {
    fn apply(&mut self, _world: &World, delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            super::ignore_incompatible!(value);
        }

        let mut sample_count: u8 = 1;
        if self.average_value.length_squared() != 0.0 {
            self.total_sample_time += delta;
            self.samples += sample_count as u32;
        }

        let mut value = value.as_axis3d();
        if delta < 0.25 {
            if self.samples > 0 && self.total_sample_time > 0.0 {
                // Seconds/sample.
                let axis_sampling_time = self.total_sample_time / self.samples as f32;
                debug_assert!(axis_sampling_time > 0.0);

                if value.length_squared() != 0.0 && sample_count > 0 {
                    self.zero_time = 0.0;
                    if self.average_value.length_squared() != 0.0 {
                        // This isn't the first tick with non-zero value.
                        if delta < axis_sampling_time * (sample_count as f32 + 1.0) {
                            // Smooth value so samples/tick is constant.
                            value *= delta / (axis_sampling_time * sample_count as f32);
                            sample_count = 1;
                        }
                    }

                    self.average_value = value * (1.0 / sample_count as f32);
                } else {
                    // No value received.
                    if self.zero_time < axis_sampling_time {
                        // Zero value is possibly because less than the value sampling interval has passed.
                        value = self.average_value * (delta / axis_sampling_time);
                    } else {
                        self.reset();
                    }

                    self.zero_time += delta; // increment length of time we've been at zero
                }
            }
        } else {
            // If we had an abnormally long frame, clear everything so it doesn't distort the results.
            self.reset();
        }

        ActionValue::Axis3D(value).convert(dim)
    }
}

impl Default for Smooth {
    fn default() -> Self {
        Self {
            zero_time: Default::default(),
            average_value: Default::default(),
            samples: Default::default(),
            total_sample_time: Self::DEFAULT_SAMPLE_TIME,
        }
    }
}

impl Smooth {
    const DEFAULT_SAMPLE_TIME: f32 = 0.0083;

    fn reset(&mut self) {
        self.zero_time = 0.0;
        self.average_value = Vec3::ZERO;
        self.samples = 0;
        self.total_sample_time = Self::DEFAULT_SAMPLE_TIME;
    }
}
