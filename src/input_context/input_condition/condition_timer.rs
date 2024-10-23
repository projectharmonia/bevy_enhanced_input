use bevy::prelude::*;

/// Helper for building triggers that have firing conditions governed by elapsed time.
#[derive(Clone, Copy, Default, Debug)]
pub struct ConditionTimer {
    /// If set to `true`, [`Time::relative_speed`] will be applied to the held duration.
    ///
    /// By default is set to `false`.
    pub relative_speed: bool,

    duration: f32,
}

impl ConditionTimer {
    pub fn update(&mut self, time: &Time<Virtual>) {
        // Time<Virtual> returns already scaled results.
        // Unscale if configured.
        let scale = if self.relative_speed {
            1.0
        } else {
            time.relative_speed()
        };

        self.duration += time.delta_seconds() / scale;
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
    use std::time::Duration;

    use super::*;

    #[test]
    fn absolute() {
        let mut time = Time::<Virtual>::default();
        time.set_relative_speed(0.5);
        time.advance_by(Duration::from_millis(200 / 2)); // Advance needs to be scaled manually.

        let mut timer = ConditionTimer::default();
        timer.update(&time);
        assert_eq!(timer.duration(), 0.2);
    }

    #[test]
    fn relative() {
        let mut time = Time::<Virtual>::default();
        time.set_relative_speed(0.5);
        time.advance_by(Duration::from_millis(200 / 2)); // Advance needs to be scaled manually.

        let mut timer = ConditionTimer {
            relative_speed: true,
            ..Default::default()
        };
        timer.update(&time);
        assert_eq!(timer.duration(), 0.1);
    }
}
