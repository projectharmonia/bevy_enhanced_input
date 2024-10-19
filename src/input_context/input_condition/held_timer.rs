use bevy::prelude::*;

/// Helper for building triggers that have firing conditions governed by elapsed time.
#[derive(Clone, Copy, Default, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative() {
        let mut time = Time::<Virtual>::default();
        time.set_relative_speed(0.5);
        let mut world = World::new();
        world.insert_resource(time);

        let mut timer = HeldTimer::relative_to_speed(true);
        timer.update(&world, 1.0);
        assert_eq!(timer.duration(), 0.5);
    }
}
