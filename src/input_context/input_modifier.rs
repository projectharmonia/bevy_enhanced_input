pub mod accumulate_by;
pub mod dead_zone;
pub mod delta_lerp;
pub mod delta_scale;
pub mod exponential_curve;
pub mod negate;
pub mod scale;
pub mod swizzle_axis;

use std::{fmt::Debug, iter};

use bevy::prelude::*;

use super::context_instance::ActionsData;
use crate::action_value::ActionValue;

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input maps to axes.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBind::with_modifiers`](super::context_instance::ActionBind::with_modifiers)
/// and [`InputBindModCond::with_modifiers`](super::input_bind::InputBindModCond::with_modifiers).
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

/// Represents collection of bindings that could be passed into
/// [`ActionBind::with_modifiers`](super::context_instance::ActionBind::with_modifiers)
/// and [`InputBindModCond::with_modifiers`](super::input_bind::InputBindModCond::with_modifiers).
pub trait InputModifiers {
    /// Returns an iterator over modifiers.
    fn iter_modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>>;
}

impl<I: InputModifier> InputModifiers for I {
    fn iter_modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>> {
        iter::once(Box::new(self) as Box<dyn InputModifier>)
    }
}

macro_rules! impl_tuple_modifiers {
    ($($name:ident),+) => {
        impl<$($name),+> InputModifiers for ($($name,)+)
        where
            $($name: InputModifier),+
        {
            #[allow(non_snake_case)]
            fn iter_modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>> {
                let ($($name,)+) = self;
                std::iter::empty()
                    $(.chain(iter::once(Box::new($name) as Box<dyn InputModifier>)))+
            }
        }
    };
}

bevy::utils::all_tuples!(impl_tuple_modifiers, 1, 15, I);
