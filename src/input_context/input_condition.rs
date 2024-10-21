pub mod blocked_by;
pub mod chord;
pub mod condition_timer;
pub mod down;
pub mod hold;
pub mod hold_and_release;
pub mod pressed;
pub mod pulse;
pub mod released;
pub mod tap;

use std::fmt::Debug;

use super::{context_instance::ActionContext, input_action::ActionState};
use crate::action_value::ActionValue;

pub const DEFAULT_ACTUATION: f32 = 0.5;

/// Defines how input activates.
///
/// Most conditions analyze the input itself, checking for minimum actuation values
/// and validating patterns like short taps, prolonged holds, or the typical "press"
/// or "release" events.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBind::with_condition`](super::context_instance::ActionBind::with_condition)
/// and [`InputBind::with_condition`](super::context_instance::InputBind::with_condition).
///
/// You can create game-specific conditions:
///
/// ```
/// # use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// #[derive(Debug, Clone, Copy)]
/// struct OnGround;
///
/// impl InputCondition for OnGround {
///     fn evaluate(&mut self, ctx: &ActionContext, _delta: f32, _value: ActionValue) -> ActionState {
///         let entity = *ctx.entities.first().unwrap();
///         let transform = ctx.world.get::<Transform>(entity).unwrap();
///         if transform.translation.z <= 0.1 {
///             ActionState::Fired
///         } else {
///             ActionState::None
///         }
///     }
///
///     fn kind(&self) -> ConditionKind {
///         ConditionKind::Required
///     }
/// }
/// ```
pub trait InputCondition: Sync + Send + Debug + 'static {
    /// Returns calculates state.
    fn evaluate(&mut self, ctx: &ActionContext, delta: f32, value: ActionValue) -> ActionState;

    /// Returns how the condition is combined with others.
    fn kind(&self) -> ConditionKind {
        ConditionKind::Regular
    }
}

/// Determines how a condition contributes to the final [`ActionState`].
pub enum ConditionKind {
    /// The most significant [`ActionState`] from all regular conditions will be the
    /// resulting state.
    ///
    /// If no regular conditions are provided, the action will be set to [`ActionState::Fired`] on
    /// any non-zero value, functioning similarly to a [`Down`](down::Down) condition with a zero actuation threshold.
    Regular,
    /// If any required condition fails to return [`ActionState::Fired`],
    /// it will override all results from regular actions with [`ActionState::None`].
    /// Doesn't contribute to the action state on its own.
    ///
    /// Useful if you want to force fail for an action or an input.
    Required,
}
