pub mod accumulate_by;
pub mod dead_zone;
pub mod delta_lerp;
pub mod delta_scale;
pub mod exponential_curve;
pub mod negate;
pub mod scale;
pub mod swizzle_axis;

use std::fmt::Debug;

use bevy::prelude::*;

use super::context_instance::ActionsData;
use crate::action_value::ActionValue;

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input maps to axes.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBind::with_modifier`](super::context_instance::ActionBind::with_modifier)
/// and [`InputBindModCond::with_modifier`](super::input_bind::InputBindModCond::with_modifier).
pub trait InputModifier: Sync + Send + Debug + 'static {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn apply(
        &mut self,
        actions: &ActionsData,
        time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue;
}
