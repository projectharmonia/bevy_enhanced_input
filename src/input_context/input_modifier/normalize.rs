use bevy::prelude::*;

use super::{ignore_incompatible, InputModifier};
use crate::action_value::{ActionValue, ActionValueDim};

/// Normalizes input if possible or returns zero.
#[derive(Clone, Copy, Debug)]
pub struct Normalize;

impl InputModifier for Normalize {
    fn apply(&mut self, _time: &Time<Virtual>, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool || dim == ActionValueDim::Axis1D {
            ignore_incompatible!(value);
        }

        let normalized = value.as_axis3d().normalize_or_zero();
        ActionValue::Axis3D(normalized).convert(dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization() {
        let time = Time::default();

        assert_eq!(Normalize.apply(&time, true.into()), true.into());
        assert_eq!(Normalize.apply(&time, 0.5.into()), 0.5.into());
        assert_eq!(Normalize.apply(&time, Vec2::ZERO.into()), Vec2::ZERO.into(),);
        assert_eq!(
            Normalize.apply(&time, Vec2::ONE.into()),
            Vec2::ONE.normalize_or_zero().into(),
        );
        assert_eq!(
            Normalize.apply(&time, Vec3::ONE.into()),
            Vec3::ONE.normalize_or_zero().into(),
        );
    }
}
