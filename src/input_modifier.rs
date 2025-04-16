pub mod accumulate_by;
pub mod dead_zone;
pub mod delta_scale;
pub mod exponential_curve;
pub mod negate;
pub mod scale;
pub mod smooth_nudge;
pub mod swizzle_axis;

use alloc::boxed::Box;
use core::{fmt::Debug, iter};

use bevy::prelude::*;

use crate::{action_map::ActionMap, action_value::ActionValue};

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input maps to axes.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBinding::with_modifiers`](crate::action_binding::ActionBinding::with_modifiers)
/// and [`BindingBuilder::with_modifiers`](crate::input_binding::BindingBuilder::with_modifiers).
pub trait InputModifier: Sync + Send + Debug + 'static {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn apply(
        &mut self,
        action_map: &ActionMap,
        time: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue;
}

/// Conversion into iterator of bindings that could be passed into
/// [`ActionBinding::with_modifiers`](crate::action_binding::ActionBinding::with_modifiers)
/// and [`BindingBuilder::with_modifiers`](crate::input_binding::BindingBuilder::with_modifiers).
pub trait IntoModifiers {
    /// Returns an iterator over modifiers.
    fn into_modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>>;
}

impl<I: InputModifier> IntoModifiers for I {
    fn into_modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>> {
        iter::once(Box::new(self) as Box<dyn InputModifier>)
    }
}

macro_rules! impl_tuple_modifiers {
    ($($name:ident),+) => {
        impl<$($name),+> IntoModifiers for ($($name,)+)
        where
            $($name: InputModifier),+
        {
            #[allow(non_snake_case)]
            fn into_modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>> {
                let ($($name,)+) = self;
                core::iter::empty()
                    $(.chain(iter::once(Box::new($name) as Box<dyn InputModifier>)))+
            }
        }
    };
}

variadics_please::all_tuples!(impl_tuple_modifiers, 1, 15, I);
