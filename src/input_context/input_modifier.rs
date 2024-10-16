pub mod dead_zone;
pub mod exponential_curve;
pub mod negate;
pub mod scalar;
pub mod scale_by_delta;
pub mod smooth_delta;
pub mod swizzle_axis;

use std::fmt::Debug;

use bevy::prelude::*;

use crate::action_value::ActionValue;

/// Pre-processors that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input behaves based on the state of the player.
///
/// Because you have access to the world when making your own modifier, you can access any game state you want.
///
/// Modifiers can be applied both to inputs and actions.
/// See [`ActionMap::with_modifier`](super::context_instance::ActionMap::with_modifier)
/// and [`InputMap::with_modifier`](super::context_instance::InputMap::with_modifier).
pub trait InputModifier: Sync + Send + Debug + 'static {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn apply(&mut self, world: &World, delta: f32, value: ActionValue) -> ActionValue;
}

/// Simple helper to emit a warning if a dimension is not compatible with a modifier.
///
/// We use a macro to make [`warn_once`] work independently.
macro_rules! ignore_incompatible {
    ($value:expr) => {
        warn_once!(
            "trying to apply `{}` to a `{:?}` value, which is not possible",
            std::any::type_name::<Self>(),
            $value.dim(),
        );
        return $value
    };
}

use ignore_incompatible;
