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

use alloc::boxed::Box;
use core::{fmt::Debug, iter};

use bevy::prelude::*;

use crate::{
    action_map::{ActionMap, ActionState},
    action_value::ActionValue,
};

/// Default actuation threshold for all conditions.
pub const DEFAULT_ACTUATION: f32 = 0.5;

/// Defines how input activates.
///
/// Conditions analyze the input, checking for minimum actuation values
/// and validating patterns like short taps, prolonged holds, or the typical "press"
/// or "release" events.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBinding::with_conditions`](crate::action_binding::ActionBinding::with_conditions)
/// and [`BindingBuilder::with_conditions`](crate::input_binding::BindingBuilder::with_conditions).
pub trait InputCondition: Sync + Send + Debug + 'static {
    /// Returns calculates state.
    ///
    /// `actions` is a state of other actions within the currently evaluating context.
    fn evaluate(
        &mut self,
        action_map: &ActionMap,
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
/// For details about how actions are combined, see [`Actions`](crate::actions::Actions).
pub enum ConditionKind {
    /// The most significant [`ActionState`] from all explicit conditions will be the
    /// resulting state.
    Explicit,
    /// Like [`Self::Explicit`], but [`ActionState::Fired`] will be set only if all
    /// implicit conditions return it.
    ///
    /// Otherwise, the most significant state will be capped at [`ActionState::Ongoing`].
    Implicit,
    /// Any blocking condition that returns [`ActionState::None`] will override
    /// the state with [`ActionState::None`] or block the events.
    ///
    /// Doesn't contribute to the state on its own.
    Blocker {
        /// Block only events instead of overriding the state.
        ///
        /// Other actions will be able to see the action state in [`ActionMap`].
        events_only: bool,
    },
}

/// Conversion into iterator of bindings that could be passed into
/// [`ActionBinding::with_conditions`](crate::action_binding::ActionBinding::with_conditions)
/// and [`BindingBuilder::with_conditions`](crate::input_binding::BindingBuilder::with_conditions).
pub trait IntoConditions {
    /// Returns an iterator over conditions.
    fn into_conditions(self) -> impl Iterator<Item = Box<dyn InputCondition>>;
}

impl<I: InputCondition> IntoConditions for I {
    fn into_conditions(self) -> impl Iterator<Item = Box<dyn InputCondition>> {
        iter::once(Box::new(self) as Box<dyn InputCondition>)
    }
}

macro_rules! impl_tuple_condition {
    ($($name:ident),+) => {
        impl<$($name),+> IntoConditions for ($($name,)+)
        where
            $($name: InputCondition),+
        {
            #[allow(non_snake_case)]
            fn into_conditions(self) -> impl Iterator<Item = Box<dyn InputCondition>> {
                let ($($name,)+) = self;
                core::iter::empty()
                    $(.chain(iter::once(Box::new($name) as Box<dyn InputCondition>)))+
            }
        }
    };
}

variadics_please::all_tuples!(impl_tuple_condition, 1, 15, I);
