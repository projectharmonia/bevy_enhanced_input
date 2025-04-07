use alloc::vec::Vec;
use core::{
    any::{self, TypeId},
    marker::PhantomData,
};

use bevy::{prelude::*, utils::Entry};

use crate::{
    action_binding::ActionBinding,
    action_map::{Action, ActionMap, ActionState},
    action_value::ActionValue,
    input::GamepadDevice,
    input_action::InputAction,
    input_reader::{InputReader, ResetInput},
};

/// Component that stores a actions with their bindings for specific [`InputContext`].
///
/// Bindings represented by [`ActionBinding`] and can be added to specific action using [`Self::bind`].
/// Data for each bound action is stored inside [`ActionMap`].
///
/// Until this component exists on the entity, actions will be evaluated and trigger [`events`](crate::events).
#[derive(Component)]
pub struct Actions<C: InputContext> {
    gamepad: GamepadDevice,
    bindings: Vec<ActionBinding>,
    action_map: ActionMap,
    marker: PhantomData<C>,
}

impl<C: InputContext> Actions<C> {
    /// Associates context with gamepad.
    ///
    /// Context will process input only from this gamepad.
    ///
    /// By default it's [`GamepadDevice::Any`].
    pub fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        self.gamepad = gamepad.into();
    }

    /// Adds action `A` to this input context and returns mutable reference to the action bindings.
    ///
    /// This method can be called multiple times for the same action to extend its mappings.
    pub fn bind<A: InputAction>(&mut self) -> &mut ActionBinding {
        let type_id = TypeId::of::<A>();
        match self.action_map.entry(type_id) {
            Entry::Occupied(_entry) => self
                .bindings
                .iter_mut()
                .find(|binding| binding.type_id() == type_id)
                .expect("actions and bindings should have matching type IDs"),
            Entry::Vacant(entry) => {
                entry.insert(Action::new::<A>());
                self.bindings.push(ActionBinding::new::<A>());
                self.bindings.last_mut().unwrap()
            }
        }
    }

    /// Returns associated bindings for action `A` if exists.
    ///
    /// For panicking version see [`Self::binding`].
    /// For assigning bindings use [`Self::bind`].
    pub fn get_binding<A: InputAction>(&self) -> Option<&ActionBinding> {
        self.bindings
            .iter()
            .find(|binding| binding.type_id() == TypeId::of::<A>())
    }

    /// Returns associated bindings for action `A`.
    ///
    /// For non-panicking version see [`Self::get_binding`].
    /// For assigning bindings use [`Self::bind`].
    pub fn binding<A: InputAction>(&self) -> &ActionBinding {
        self.get_binding::<A>().unwrap_or_else(|| {
            panic!(
                "action `{}` should be binded before access",
                any::type_name::<A>()
            )
        })
    }

    /// Returns associated state for action `A` if exists.
    ///
    /// For panicking version see [`Self::action`].
    pub fn get_action<A: InputAction>(&self) -> Option<&Action> {
        self.action_map.action::<A>()
    }

    /// Returns associated state for action `A`.
    ///
    /// For non-panicking version see [`Self::get_action`].
    ///
    /// # Panics
    ///
    /// Panics if the action `A` was not bound beforehand.
    pub fn action<A: InputAction>(&self) -> &Action {
        self.get_action::<A>().unwrap_or_else(|| {
            panic!(
                "action `{}` should be binded before access",
                any::type_name::<A>()
            )
        })
    }

    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        reader.set_gamepad(self.gamepad);
        for binding in &mut self.bindings {
            binding.update(commands, reader, &mut self.action_map, time, entity);
        }
    }

    /// Sets the state for each action to [`ActionState::None`]  and triggers transitions with zero value.
    ///
    /// Resets the input.
    pub(crate) fn reset(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        for binding in self.bindings.drain(..) {
            let action = self
                .action_map
                .get_mut(&binding.type_id())
                .expect("actions and bindings should have matching type IDs");
            action.update(time, ActionState::None, ActionValue::zero(binding.dim()));
            action.trigger_events(commands, entity);
            if binding.require_reset() {
                reset_input.extend(binding.inputs().iter().map(|binding| binding.input));
            }
        }

        self.gamepad = Default::default();
        self.action_map.clear();
    }
}

impl<C: InputContext> Default for Actions<C> {
    fn default() -> Self {
        Self {
            gamepad: Default::default(),
            bindings: Default::default(),
            action_map: Default::default(),
            marker: PhantomData,
        }
    }
}

/// Marker for a gameplay-related input context that a player can be in.
///
/// Used to differentiate [`Actions`] components and configure how associated actions will be evaluated.
///
/// All structs that implement this trait need to be registered using
/// [`InputContextAppExt::add_input_context`](crate::action_instances::InputContextAppExt::add_input_context)
///
/// # Examples
///
/// To implement the trait you can use the [`InputContext`](bevy_enhanced_input_macros::InputContext)
/// derive to reduce boilerplate:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(InputContext)]
/// struct Player;
/// ```
///
/// Optionally you can pass `priority`:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(InputContext)]
/// #[input_context(priority = 1)]
/// struct Player;
/// ```
///
/// All parameters match corresponding data in the trait.
pub trait InputContext: Send + Sync + 'static {
    /// Determines the evaluation order of [`Actions<Self>`].
    ///
    /// Used to control how contexts are layered since some [`InputAction`]s may consume inputs.
    ///
    /// Ordering is global. Contexts with a higher priority evaluated first.
    const PRIORITY: usize = 0;
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::{InputAction, InputContext};

    use super::*;

    #[test]
    fn bind() {
        let mut actions = Actions::<Dummy>::default();
        actions.bind::<DummyAction>().to(KeyCode::KeyA);
        actions.bind::<DummyAction>().to(KeyCode::KeyB);
        assert_eq!(actions.bindings.len(), 1);

        let binding = actions.binding::<DummyAction>();
        assert_eq!(binding.inputs().len(), 2);
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;

    #[derive(InputContext)]
    struct Dummy;
}
