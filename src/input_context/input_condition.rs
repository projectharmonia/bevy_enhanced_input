pub mod block_by;
pub mod chord;
pub mod down;
pub mod hold;
pub mod hold_and_release;
pub mod press;
pub mod pulse;
pub mod release;
pub mod tap;

use alloc::boxed::Box;
use core::{fmt::Debug, iter};

use bevy::utils::TypeIdMap;

use crate::prelude::*;

/// Default actuation threshold for all conditions.
pub const DEFAULT_ACTUATION: f32 = 0.5;

/// Defines how input activates.
///
/// Conditions analyze the input, checking for minimum actuation values
/// and validating patterns like short taps, prolonged holds, or the typical "press"
/// or "release" events.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBinding::with_conditions`] and [`BindingBuilder::with_conditions`].
pub trait InputCondition: Sync + Send + Debug + 'static {
    /// Returns calculates state.
    ///
    /// `actions` is a state of other actions within the currently evaluating context.
    fn evaluate(
        &mut self,
        action_map: &TypeIdMap<UntypedAction>,
        time: &InputTime,
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
/// on any non-zero value, functioning similarly to a [`Down`] condition
/// with a zero actuation threshold.
///
/// For details about how actions are combined, see [`Actions`].
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
    /// the state with [`ActionState::None`].
    ///
    /// Doesn't contribute to the state on its own.
    Blocker,
}

/// Conversion into iterator of bindings that could be passed into
/// [`ActionBinding::with_conditions`] and [`BindingBuilder::with_conditions`].
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
