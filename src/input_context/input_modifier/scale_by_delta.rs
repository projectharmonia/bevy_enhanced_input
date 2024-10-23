use bevy::prelude::*;

use super::{ignore_incompatible, InputModifier};
use crate::action_value::{ActionValue, ActionValueDim};

/// Multiplies the input value by delta time for this frame.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct ScaleByDelta;

impl InputModifier for ScaleByDelta {
    fn apply(&mut self, time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            ignore_incompatible!(value);
        }

        ActionValue::Axis3D(value.as_axis3d() * time.delta_seconds()).convert(dim)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn scaling() {
        let mut time = Time::default();
        time.advance_by(Duration::from_millis(500));

        assert_eq!(ScaleByDelta.apply(&time, true.into()), true.into());
        assert_eq!(ScaleByDelta.apply(&time, 0.5.into()), 0.25.into());
        assert_eq!(
            ScaleByDelta.apply(&time, Vec2::ONE.into()),
            (0.5, 0.5).into()
        );
        assert_eq!(
            ScaleByDelta.apply(&time, Vec3::ONE.into()),
            (0.5, 0.5, 0.5).into()
        );
    }
}
