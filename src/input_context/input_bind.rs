use std::iter;

use super::{
    input_condition::{InputCondition, InputConditionSet},
    input_modifier::{InputModifier, InputModifierSet},
};
use crate::input::Input;

/// Associated input for [`ActionBind`](super::context_instance::ActionBind).
#[derive(Debug)]
pub struct InputBind {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,

    /// Newly created mappings are ignored by default until until a zero
    /// value is read for them.
    ///
    /// This prevents newly created contexts from reacting to currently
    /// held inputs until they are released.
    pub(super) ignored: bool,
}

impl InputBind {
    /// Creates a new instance without modifiers and conditions.
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            modifiers: Default::default(),
            conditions: Default::default(),
            ignored: true,
        }
    }
}

impl<I: Into<Input>> From<I> for InputBind {
    fn from(input: I) -> Self {
        Self::new(input)
    }
}

/// A trait to ergonomically add modifiers or conditions to any type that can be converted into a binding.
pub trait InputBindModCond {
    /// Adds modifiers.
    #[must_use]
    fn with_modifiers(self, set: impl InputModifierSet) -> InputBind;

    /// Adds conditions.
    #[must_use]
    fn with_conditions(self, set: impl InputConditionSet) -> InputBind;
}

impl<T: Into<InputBind>> InputBindModCond for T {
    fn with_modifiers(self, set: impl InputModifierSet) -> InputBind {
        let mut binding = self.into();
        binding.modifiers.extend(set.modifiers());
        binding
    }

    fn with_conditions(self, set: impl InputConditionSet) -> InputBind {
        let mut binding = self.into();
        binding.conditions.extend(set.conditions());
        binding
    }
}

/// Represents collection of bindings that could be passed into
/// [`ActionBind::to`](super::context_instance::ActionBind::to).
///
/// Can be manually implemented to provide custom modifiers or conditions.
/// See [`preset`](super::preset) for examples.
pub trait InputBindSet {
    /// Returns an iterator over bindings.
    fn bindings(self) -> impl Iterator<Item = InputBind>;
}

impl<I: Into<InputBind>> InputBindSet for I {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        iter::once(self.into())
    }
}

impl<I: Into<InputBind> + Copy> InputBindSet for &Vec<I> {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.as_slice().bindings()
    }
}

impl<I: Into<InputBind> + Copy, const N: usize> InputBindSet for &[I; N] {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.as_slice().bindings()
    }
}

impl<I: Into<InputBind> + Copy> InputBindSet for &[I] {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.iter().copied().map(Into::into)
    }
}

macro_rules! impl_tuple_binds {
    ($($name:ident),+) => {
        impl<$($name),+> InputBindSet for ($($name,)+)
        where
            $($name: InputBindSet),+
        {
            #[allow(non_snake_case)]
            fn bindings(self) -> impl Iterator<Item = InputBind> {
                let ($($name,)+) = self;
                std::iter::empty()
                    $(.chain($name.bindings()))+
            }
        }
    };
}

bevy::utils::all_tuples!(impl_tuple_binds, 1, 15, I);
