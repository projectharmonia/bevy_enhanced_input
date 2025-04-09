use alloc::{boxed::Box, vec::Vec};
use core::iter;

use crate::{
    input::Input,
    input_condition::{InputCondition, IntoConditions},
    input_modifier::{InputModifier, IntoModifiers},
};

/// Associated input for [`ActionBinding`](crate::action_binding::ActionBinding).
#[derive(Debug)]
pub struct InputBinding {
    pub input: Input,
    pub modifiers: Vec<Box<dyn InputModifier>>,
    pub conditions: Vec<Box<dyn InputCondition>>,

    /// Whether the input output a non-zero value.
    ///
    /// Prevents newly created contexts from reacting to currently held inputs
    /// until they’re released.
    ///
    /// Used only if [`ActionBinding`](crate::action_binding::ActionBinding::require_reset) is set.
    pub(crate) first_activation: bool,
}

impl InputBinding {
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

impl<I: Into<Input>> From<I> for InputBinding {
    fn from(input: I) -> Self {
        Self::new(input)
    }
}

/// A trait to ergonomically add modifiers or conditions to any type that can be converted into a binding.
pub trait BindingBuilder {
    /// Adds input-level modifiers.
    ///
    /// For action-level conditions see
    /// [`ActionBinding::with_modifiers`](crate::action_binding::ActionBinding::with_modifiers).
    ///
    /// # Examples
    ///
    /// Single modifier:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space.with_modifiers(Scale::splat(2.0)));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space.with_modifiers((Scale::splat(2.0), Negate::all())));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Jump;
    /// ```
    #[must_use]
    fn with_modifiers(self, modifiers: impl IntoModifiers) -> InputBinding;

    /// Adds input-level conditions.
    ///
    /// You can also apply modifiers to multiple inputs using [`IntoBindings::with_modifiers_each`]
    ///
    /// For action-level conditions see
    /// [`ActionBinding::with_conditions`](crate::action_binding::ActionBinding::with_conditions).
    ///
    /// # Examples
    ///
    /// Single condition:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space.with_conditions(Release::default()));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space.with_conditions((Release::default(), JustPress::default())));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// # #[derive(InputContext)]
    /// # struct Dummy;
    /// ```
    #[must_use]
    fn with_conditions(self, conditions: impl IntoConditions) -> InputBinding;
}

impl<T: Into<InputBinding>> BindingBuilder for T {
    fn with_modifiers(self, modifiers: impl IntoModifiers) -> InputBinding {
        let mut binding = self.into();
        binding.modifiers.extend(modifiers.into_modifiers());
        binding
    }

    fn with_conditions(self, conditions: impl IntoConditions) -> InputBinding {
        let mut binding = self.into();
        binding.conditions.extend(conditions.into_conditions());
        binding
    }
}

/// Conversion into iterator of bindings that could be passed into
/// [`ActionBinding::to`](crate::action_binding::ActionBinding::to).
///
/// Can be manually implemented to provide custom modifiers or conditions.
/// See [`preset`](crate::preset) for examples.
pub trait IntoBindings {
    /// Returns an iterator over bindings.
    fn into_bindings(self) -> impl Iterator<Item = InputBinding>;

    /// Adds modifiers to **each** binding.
    ///
    /// <div class="warning">
    ///
    /// Avoid using this with modifiers like [`DeadZone`](crate::input_modifier::dead_zone::DeadZone),
    /// as this method applies the modifier to each input **individually** rather than to all bindings.
    ///
    /// </div>
    ///
    /// # Examples
    ///
    /// Negate each gamepad axis for the stick:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Move>()
    ///     .to((
    ///         Input::mouse_motion(),
    ///         GamepadStick::Left.with_modifiers_each(Negate::all()), // Will be applied to each axis.
    ///     ))
    ///     .with_modifiers(DeadZone::default()); // Modifiers like `DeadZone` need to be applied at the action level!
    /// # #[derive(InputContext)]
    /// # struct Dummy;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Move;
    /// ```
    fn with_modifiers_each<M: IntoModifiers + Clone>(
        self,
        modifiers: M,
    ) -> WithModifiersEach<Self, M>
    where
        Self: Sized,
    {
        WithModifiersEach {
            bindings: self,
            modifiers,
        }
    }

    /// Adds condition to **each** binding.
    ///
    /// Similar to [`Self::with_modifiers_each`].
    fn with_conditions_each<C: IntoConditions + Clone>(
        self,
        conditions: C,
    ) -> WithConditionsEach<Self, C>
    where
        Self: Sized,
    {
        WithConditionsEach {
            bindings: self,
            conditions,
        }
    }
}

impl<I: Into<InputBinding>> IntoBindings for I {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        iter::once(self.into())
    }
}

impl<I: Into<InputBinding> + Copy> IntoBindings for &Vec<I> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        self.as_slice().into_bindings()
    }
}

impl<I: Into<InputBinding> + Copy, const N: usize> IntoBindings for &[I; N] {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        self.as_slice().into_bindings()
    }
}

impl<I: Into<InputBinding> + Copy> IntoBindings for &[I] {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        self.iter().copied().map(Into::into)
    }
}

macro_rules! impl_tuple_binds {
    ($($name:ident),+) => {
        impl<$($name),+> IntoBindings for ($($name,)+)
        where
            $($name: IntoBindings),+
        {
            #[allow(non_snake_case)]
            fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
                let ($($name,)+) = self;
                core::iter::empty()
                    $(.chain($name.into_bindings()))+
            }
        }
    };
}

variadics_please::all_tuples!(impl_tuple_binds, 1, 15, I);

/// Bindings with assigned modifiers.
///
/// See also [`IntoBindings::with_modifiers_each`]
pub struct WithModifiersEach<I: IntoBindings, M: IntoModifiers + Clone> {
    bindings: I,
    modifiers: M,
}

impl<I: IntoBindings, M: IntoModifiers + Clone> IntoBindings for WithModifiersEach<I, M> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        self.bindings
            .into_bindings()
            .map(move |binding| binding.with_modifiers(self.modifiers.clone()))
    }
}

/// Bindings with assigned conditions.
///
/// See also [`IntoBindings::with_conditions_each`]
pub struct WithConditionsEach<I: IntoBindings, C: IntoConditions + Clone> {
    bindings: I,
    conditions: C,
}

impl<I: IntoBindings, C: IntoConditions + Clone> IntoBindings for WithConditionsEach<I, C> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        self.bindings
            .into_bindings()
            .map(move |binding| binding.with_conditions(self.conditions.clone()))
    }
}
