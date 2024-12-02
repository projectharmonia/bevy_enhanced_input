pub mod block_by;
pub mod chord;
pub mod condition_timer;
pub mod hold;
pub mod hold_and_release;
pub mod just_press;
pub mod press;
pub mod pulse;
pub mod release;
pub mod tap;

use std::{fmt::Debug, iter};

use bevy::prelude::*;

use super::context_instance::{ActionState, ActionsData};
use crate::action_value::ActionValue;

pub const DEFAULT_ACTUATION: f32 = 0.5;

/// Defines how input activates.
///
/// Conditions analyze the input, checking for minimum actuation values
/// and validating patterns like short taps, prolonged holds, or the typical "press"
/// or "release" events.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBind::with_conditions`](super::context_instance::ActionBind::with_conditions)
/// and [`InputBindModCond::with_conditions`](super::input_bind::InputBindModCond::with_conditions).
pub trait InputCondition: Sync + Send + Debug + 'static {
    /// Returns calculates state.
    ///
    /// `actions` argument a state of other actions within the currently evaluating context.
    fn evaluate(
        &mut self,
        actions: &ActionsData,
        time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionState;

    /// Returns how the condition is combined with others.
    fn kind(&self) -> ConditionKind {
        ConditionKind::Explicit
    }
}

/// Determines how a condition contributes to the final [`ActionState`].
///
/// If no conditions are provided, the state will be set to [`ActionState::Fired`]
/// on any non-zero value, functioning similarly to a [`Press`](press::Press) condition
/// with a zero actuation threshold.
///
/// For details about how actions are combined, see [`ContextInstance`](super::context_instance::ContextInstance).
pub enum ConditionKind {
    /// The most significant [`ActionState`] from all explicit conditions will be the
    /// resulting state.
    Explicit,
    /// Like [`Self::Explicit`], but [`ActionState::Fired`] will be set only if all
    /// implicit conditions return it.
    ///
    /// Otherwise, the most significant state will be capped at [`ActionState::Ongoing`].
    Implicit,
    /// If any blocking condition fails to return [`ActionState::Fired`],
    /// it will override the state with [`ActionState::None`] or block the events.
    ///
    /// Doesn't contribute to the state on its own.
    Blocker {
        /// Block only events instead of overriding the state.
        ///
        /// Other actions will be able to see the action state in [`ActionsData`].
        events_only: bool,
    },
}

/// Represents collection of bindings that could be passed into
/// [`ActionBind::with_conditions`](super::context_instance::ActionBind::with_conditions)
/// and [`InputBindModCond::with_conditions`](super::input_bind::InputBindModCond::with_conditions).
pub trait InputConditionSet {
    /// Returns an iterator over conditions.
    fn conditions(self) -> impl Iterator<Item = Box<dyn InputCondition>>;
}

impl<I: InputCondition> InputConditionSet for I {
    fn conditions(self) -> impl Iterator<Item = Box<dyn InputCondition>> {
        iter::once(Box::new(self) as Box<dyn InputCondition>)
    }
}

macro_rules! impl_tuple_condition {
    ($($name:ident),+) => {
        impl<$($name),+> InputConditionSet for ($($name,)+)
        where
            $($name: InputCondition),+
        {
            #[allow(non_snake_case)]
            fn conditions(self) -> impl Iterator<Item = Box<dyn InputCondition>> {
                let ($($name,)+) = self;
                std::iter::empty()
                    $(.chain(iter::once(Box::new($name) as Box<dyn InputCondition>)))+
            }
        }
    };
}

bevy::utils::all_tuples!(impl_tuple_condition, 1, 15, I);
