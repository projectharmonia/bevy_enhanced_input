use bevy::prelude::*;

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
