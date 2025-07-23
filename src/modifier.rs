pub mod accumulate_by;
pub mod clamp;
pub mod dead_zone;
pub mod delta_scale;
pub mod exponential_curve;
pub mod fns;
pub mod linear_step;
pub mod negate;
pub mod scale;
pub mod smooth_nudge;
pub mod swizzle_axis;

use core::fmt::Debug;

use crate::prelude::*;

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input maps to axes.
///
/// Can be attached both to bindings and actions.
///
/// If you create a custom modifier, it needs to be registered using
/// [`InputModifierAppExt::add_input_modifier`].
pub trait InputModifier: Debug {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn transform(
        &mut self,
        actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue;
}
