use crate::{action_map::ActionMap, prelude::*};

/// Multiplies the input value by delta time for this frame.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Clone, Copy, Debug)]
pub struct DeltaScale;

impl InputModifier for DeltaScale {
    fn apply(
        &mut self,
        _action_map: &ActionMap,
        time: &InputTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                (value * time.delta_secs()).into()
            }
            ActionValue::Axis1D(value) => (value * time.delta_secs()).into(),
            ActionValue::Axis2D(value) => (value * time.delta_secs()).into(),
            ActionValue::Axis3D(value) => (value * time.delta_secs()).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use bevy::prelude::*;

    use super::*;
    use crate::input_time;

    #[test]
    fn scaling() {
        let action_map = ActionMap::default();
        let (mut world, mut state) = input_time::init_world();
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(500));
        let time = state.get(&world);

        assert_eq!(
            DeltaScale.apply(&action_map, &time, true.into()),
            0.5.into()
        );
        assert_eq!(
            DeltaScale.apply(&action_map, &time, false.into()),
            0.0.into()
        );
        assert_eq!(
            DeltaScale.apply(&action_map, &time, 0.5.into()),
            0.25.into()
        );
        assert_eq!(
            DeltaScale.apply(&action_map, &time, Vec2::ONE.into()),
            (0.5, 0.5).into()
        );
        assert_eq!(
            DeltaScale.apply(&action_map, &time, Vec3::ONE.into()),
            (0.5, 0.5, 0.5).into()
        );
    }
}
