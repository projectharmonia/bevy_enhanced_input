use bevy::prelude::*;

use crate::prelude::*;

/// Scales input independently along each axis by a specified factor.
///
/// [`ActionValue::Bool`] will be converted into [`ActionValue::Axis1D`].
#[derive(Component, Debug, Clone, Copy)]
pub struct Scale {
    /// The factor applied to the input value.
    ///
    /// For example, if the factor is set to `Vec3::new(2.0, 2.0, 2.0)`, each input axis will be multiplied by 2.0.
    pub factor: Vec3,
}

impl Scale {
    /// Creates a new instance with all axes set to `value`.
    #[must_use]
    pub fn splat(value: f32) -> Self {
        Self::new(Vec3::splat(value))
    }

    #[must_use]
    pub fn new(factor: Vec3) -> Self {
        Self { factor }
    }
}

impl InputModifier for Scale {
    fn apply(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                (value * self.factor.x).into()
            }
            ActionValue::Axis1D(value) => (value * self.factor.x).into(),
            ActionValue::Axis2D(value) => (value * self.factor.xy()).into(),
            ActionValue::Axis3D(value) => (value * self.factor).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn scaling() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = Scale::splat(2.0);
        assert_eq!(modifier.apply(&actions, &time, true.into()), 2.0.into());
        assert_eq!(modifier.apply(&actions, &time, false.into()), 0.0.into());
        assert_eq!(modifier.apply(&actions, &time, 1.0.into()), 2.0.into());
        assert_eq!(
            modifier.apply(&actions, &time, Vec2::ONE.into()),
            (2.0, 2.0).into()
        );
        assert_eq!(
            modifier.apply(&actions, &time, Vec3::ONE.into()),
            (2.0, 2.0, 2.0).into()
        );
    }
}
