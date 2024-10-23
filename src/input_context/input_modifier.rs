pub mod dead_zone;
pub mod exponential_curve;
pub mod negate;
pub mod normalize;
pub mod scalar;
pub mod scale_by_delta;
pub mod smooth_delta;
pub mod swizzle_axis;

use std::fmt::Debug;

use bevy::prelude::*;

use crate::action_value::ActionValue;

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input maps to axes.
///
/// Modifiers should preserve the original value dimention.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBind::with_modifier`](super::context_instance::ActionBind::with_modifier)
/// and [`InputBind::with_modifier`](super::context_instance::InputBind::with_modifier).
pub trait InputModifier: Sync + Send + Debug + 'static {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn apply(&mut self, time: &Time<Virtual>, value: ActionValue) -> ActionValue;
}

/// Simple helper to emit a warning if a dimension is not compatible with a modifier.
///
/// We use a macro to make [`warn_once`](bevy::log::warn_once) print independently.
#[macro_export]
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

pub use ignore_incompatible;
