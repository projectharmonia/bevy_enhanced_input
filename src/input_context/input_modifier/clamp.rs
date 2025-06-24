use bevy::{prelude::*, utils::TypeIdMap};

use crate::prelude::*;

/// Restricts input to a certain interval independently along each axis.
///
/// [`ActionValue::Bool`] will be converted into [`ActionValue::Axis1D`] before clamping.
///
/// # Examples
///
/// Bind only positive or negative direction of [`GamepadAxis::LeftStickY`].
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn bind(
///     trigger: Trigger<Bind<Ui>>,
///     mut ui: Query<&mut Actions<Ui>>,
/// ) {
///     let mut actions = ui.get_mut(trigger.target()).unwrap();
///     actions
///         .bind::<Up>()
///         .to(GamepadAxis::LeftStickY)
///         .with_modifiers(Clamp::pos());
///     actions
///         .bind::<Down>()
///         .to(GamepadAxis::LeftStickY)
///         .with_modifiers(Clamp::neg());
/// }
///
/// #[derive(InputContext)]
/// struct Ui;
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = bool)]
/// struct Up;
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = bool)]
/// struct Down;
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Clamp {
    /// Minimum value per axis.
    pub min: Vec3,
    /// Maximum value per axis.
    pub max: Vec3,
}

impl Clamp {
    /// Creates a new instance that restricts all axes only to positive numbers.
    ///
    /// Any negative values will become 0.0.
    #[must_use]
    pub fn pos() -> Self {
        Self::splat(0.0, f32::MAX)
    }

    /// Creates a new instance that restricts all axes only to negative numbers.
    ///
    /// Any positive values will become 0.0.
    #[must_use]
    pub fn neg() -> Self {
        Self::splat(f32::MIN, 0.0)
    }

    /// Creates a new instance with all axes set to `min` and `max`.
    #[must_use]
    pub fn splat(min: f32, max: f32) -> Self {
        Self::new(Vec3::splat(min), Vec3::splat(max))
    }

    #[must_use]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

impl InputModifier for Clamp {
    fn apply(
        &mut self,
        _action_map: &TypeIdMap<UntypedAction>,
        _time: &InputTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value: f32 = if value { 1.0 } else { 0.0 };
                value.clamp(self.min.x, self.max.x).into()
            }
            ActionValue::Axis1D(value) => value.clamp(self.min.x, self.max.x).into(),
            ActionValue::Axis2D(value) => value.clamp(self.min.xy(), self.max.xy()).into(),
            ActionValue::Axis3D(value) => value.clamp(self.min, self.max).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_time;

    #[test]
    fn clamping() {
        let mut modifier = Clamp::splat(0.0, 1.0);
        let action_map = TypeIdMap::<UntypedAction>::default();
        let (world, mut state) = input_time::init_world();
        let time = state.get(&world);

        assert_eq!(modifier.apply(&action_map, &time, true.into()), 1.0.into());
        assert_eq!(modifier.apply(&action_map, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&action_map, &time, 2.0.into()), 1.0.into());
        assert_eq!(
            modifier.apply(&action_map, &time, (-1.0).into()),
            0.0.into()
        );

        assert_eq!(
            modifier.apply(&action_map, &time, Vec2::new(-1.0, 2.0).into()),
            Vec2::new(0.0, 1.0).into()
        );

        assert_eq!(
            modifier.apply(&action_map, &time, Vec3::new(-2.0, 0.5, 3.0).into()),
            Vec3::new(0.0, 0.5, 1.0).into()
        );
    }
}
