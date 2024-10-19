pub mod blocked_by;
pub mod chord;
pub mod down;
pub mod held_timer;
pub mod hold;
pub mod hold_and_release;
pub mod pressed;
pub mod pulse;
pub mod released;
pub mod tap;

use std::fmt::Debug;

use bevy::prelude::*;

use super::input_action::{ActionState, ActionsData};
use crate::action_value::ActionValue;

pub const DEFAULT_ACTUATION: f32 = 0.5;

/// Defines how input activates.
///
/// Most conditions analyze the input itself, checking for minimum actuation values
/// and validating patterns like short taps, prolonged holds, or the typical "press"
/// or "release" events.
///
/// You can define your own conditions based on the world state.
///
/// Conditions can be applied both to inputs and actions.
/// See [`ActionBind::with_condition`](super::context_instance::ActionBind::with_condition)
/// and [`InputBind::with_condition`](super::context_instance::InputBind::with_condition).
pub trait InputCondition: Sync + Send + Debug + 'static {
    /// Returns calculates state.
    fn evaluate(
        &mut self,
        world: &World,
        actions: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState;

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
