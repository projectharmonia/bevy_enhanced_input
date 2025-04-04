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

use crate::{action_value::ActionValue, actions::ActionsData};

/// Pre-processor that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input maps to axes.
///
/// Can be applied both to inputs and actions.
/// See [`ActionBind::with_modifiers`](super::actions::ActionBind::with_modifiers)
/// and [`InputBindModCond::with_modifiers`](super::input_binding::InputBindModCond::with_modifiers).
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
/// [`ActionBind::with_modifiers`](super::actions::ActionBind::with_modifiers)
/// and [`InputBindModCond::with_modifiers`](super::input_binding::InputBindModCond::with_modifiers).
pub trait InputModifierSet {
    /// Returns an iterator over modifiers.
    fn modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>>;
}

impl<I: InputModifier> InputModifierSet for I {
    fn modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>> {
        iter::once(Box::new(self) as Box<dyn InputModifier>)
    }
}

macro_rules! impl_tuple_modifiers {
    ($($name:ident),+) => {
        impl<$($name),+> InputModifierSet for ($($name,)+)
        where
            $($name: InputModifier),+
        {
            #[allow(non_snake_case)]
            fn modifiers(self) -> impl Iterator<Item = Box<dyn InputModifier>> {
                let ($($name,)+) = self;
                core::iter::empty()
                    $(.chain(iter::once(Box::new($name) as Box<dyn InputModifier>)))+
            }
        }
    };
}

bevy::utils::all_tuples!(impl_tuple_modifiers, 1, 15, I);
