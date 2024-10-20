pub mod dead_zone;
pub mod exponential_curve;
pub mod negate;
pub mod normalize;
pub mod scalar;
pub mod scale_by_delta;
pub mod smooth_delta;
pub mod swizzle_axis;

use std::fmt::Debug;

use super::context_instance::ActionContext;
use crate::action_value::ActionValue;

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input behaves based on the state of the player.
///
/// Modifiers should preserve the original value dimention.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBind::with_modifier`](super::context_instance::ActionBind::with_modifier)
/// and [`InputBind::with_modifier`](super::context_instance::InputBind::with_modifier).
///
/// You can create game-specific modifiers:
///
/// ```
/// # use bevy::prelude::*;
/// use bevy_enhanced_input::{ignore_incompatible, prelude::*};
///
/// /// Input modifier that applies sensitivity from [`Settings`].
/// #[derive(Debug, Clone, Copy)]
/// struct Sensetivity;
///
/// impl InputModifier for Sensetivity {
///     fn apply(&mut self, ctx: &ActionContext, _delta: f32, value: ActionValue) -> ActionValue {
///         let dim = value.dim();
///         if dim == ActionValueDim::Bool {
///             ignore_incompatible!(value);
///         }
///
///         let settings = ctx.world.resource::<Settings>();
///         ActionValue::Axis3D(value.as_axis3d() * settings.sensitivity).convert(dim)
///     }
/// }
///
/// #[derive(Resource)]
/// struct Settings {
///     sensitivity: f32,
/// }
/// ```
pub trait InputModifier: Sync + Send + Debug + 'static {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn apply(&mut self, ctx: &ActionContext, delta: f32, value: ActionValue) -> ActionValue;
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
