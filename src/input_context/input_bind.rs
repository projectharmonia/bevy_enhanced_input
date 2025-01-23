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

    /// Whether the input output a non-zero value.
    ///
    /// Needed to prevent newly created contexts from reacting to currently
    /// held inputs until they are released.
    ///
    /// Used only if [`ActionBind`](super::context_instance::ActionBind::require_reset) is set.
    pub(super) first_activation: bool,
}

impl InputBind {
    /// Creates a new instance without modifiers and conditions.
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
            modifiers: Default::default(),
            conditions: Default::default(),
            first_activation: true,
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
    /// Adds input-level modifiers.
    ///
    /// For action-level conditions see
    /// [`ActionBind::with_modifiers`](super::context_instance::ActionBind::with_modifiers).
    ///
    /// # Examples
    ///
    /// Single modifier:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_modifiers(Scale::splat(2.0)));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Jump;
    /// ```
    ///
    /// Multiple modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_modifiers((Scale::splat(2.0), Negate::all())));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Jump;
    /// ```
    #[must_use]
    fn with_modifiers(self, set: impl InputModifierSet) -> InputBind;

    /// Adds input-level conditions.
    ///
    /// For action-level conditions see
    /// [`ActionBind::with_conditions`](super::context_instance::ActionBind::with_conditions).
    ///
    /// # Examples
    ///
    /// Single condition:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_conditions(Release::default()));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// Multiple conditions:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space.with_conditions((Release::default(), JustPress::default())));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
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

    /// Adds modifiers to **each** binding in a set.
    ///
    /// <div class="warning">
    ///
    /// Avoid using this with modifiers like [`DeadZone`](super::input_modifier::dead_zone::DeadZone),
    /// as this method applies the modifier to each input **individually** rather than to the entire set.
    ///
    /// </div>
    ///
    /// # Examples
    ///
    /// Negate each gamepad axis for the stick set:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Move>()
    ///     .to((
    ///         Input::mouse_motion(),
    ///         GamepadStick::Left.with_modifiers_each(Negate::all()), // Will be applied to each axis.
    ///     ))
    ///     .with_modifiers(DeadZone::default()); // Modifiers like `DeadZone` need to be applied at the action level!
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Move;
    /// ```
    fn with_modifiers_each<I: InputModifierSet + Clone>(
        self,
        set: I,
    ) -> InputBindModifierEach<Self, I>
    where
        Self: Sized,
    {
        InputBindModifierEach {
            input_set: self,
            modifier_set: set,
        }
    }

    /// Adds condition to **each** binding in a set.
    ///
    /// Similar to [`Self::with_modifiers_each`].
    fn with_conditions_each<I: InputConditionSet + Clone>(
        self,
        set: I,
    ) -> InputBindConditionEach<Self, I>
    where
        Self: Sized,
    {
        InputBindConditionEach {
            input_set: self,
            condition_set: set,
        }
    }
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

/// A set with assigned modifiers.
///
/// See also [`InputBindSet::with_modifiers_each`]
pub struct InputBindModifierEach<I: InputBindSet, M: InputModifierSet + Clone> {
    input_set: I,
    modifier_set: M,
}

impl<I: InputBindSet, M: InputModifierSet + Clone> InputBindSet for InputBindModifierEach<I, M> {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.input_set
            .bindings()
            .map(move |binding| binding.with_modifiers(self.modifier_set.clone()))
    }
}

/// A set with assigned conditions.
///
/// See also [`InputBindSet::with_conditions_each`]
pub struct InputBindConditionEach<I: InputBindSet, M: InputConditionSet + Clone> {
    input_set: I,
    condition_set: M,
}

impl<I: InputBindSet, M: InputConditionSet + Clone> InputBindSet for InputBindConditionEach<I, M> {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.input_set
            .bindings()
            .map(move |binding| binding.with_conditions(self.condition_set.clone()))
    }
}
