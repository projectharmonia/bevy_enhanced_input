use alloc::vec::Vec;
use bevy::{platform::collections::hash_map::Entry, prelude::*, utils::TypeIdMap};
use core::ops::{Deref, DerefMut};
use core::{
    any::{self, TypeId},
    cmp::Reverse,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};
use log::debug;

use crate::{
    input_reader::{InputReader, ResetInput},
    prelude::*,
};

/// Component that stores actions with their bindings for a specific [`InputContext`].
///
/// Bindings represented by [`ActionBinding`] and can be added to specific action using [`UntypedActions::bind`].
/// Data for each bound action represented by [`Action`].
///
/// Actions are evaluated and trigger [`events`](super::events) only when this component exists on an entity.
#[derive(Component)]
pub struct Actions<C: InputContext> {
    actions: UntypedActions,
    marker: PhantomData<C>,
}

impl<C: InputContext> Deref for Actions<C> {
    type Target = UntypedActions;

    fn deref(&self) -> &Self::Target {
        &self.actions
    }
}

impl<C: InputContext> DerefMut for Actions<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.actions
    }
}

/// Data associated with an [`InputContext`] marker.
///
/// Type-erased version of [`Actions`] stored inside it.
#[derive(Default)]
pub struct UntypedActions {
    gamepad: GamepadDevice,
    bindings: Vec<ActionBinding>,
    action_map: TypeIdMap<UntypedAction>,
    sort_required: bool,
}

impl UntypedActions {
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
        self.sort_required = true;
        self.get_or_create_binding::<A>()
    }

    /// Like [`Self::mock`], but sets the value only for a single context evaluation.
    pub fn mock_once<A: InputAction>(&mut self, state: ActionState, value: A::Output) {
        self.mock::<A>(state, value, MockSpan::Updates(1));
    }

    /// Mocks the state and value of a specific action for a given span.
    ///
    /// While mocked, the action will skip input evaluation, conditions, and modifiers,
    /// and instead report the provided state and value. All state transition events will be
    /// triggered as usual.
    ///
    /// Once the span expires, the action will resume evaluating real input.
    ///
    /// The action will be created if it doesn't already exist in the context, so calling
    /// [`Self::bind`] beforehand is not required.
    ///
    /// Mocking won't take effect immediately - it will be applied on the next context evaluation.
    /// See [`InputContext::Schedule`] for details.
    ///
    /// # Examples
    ///
    /// Move up for 2 seconds:
    ///
    /// ```
    /// # use core::time::Duration;
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.mock::<Move>(ActionState::Fired, Vec2::Y, Duration::from_secs(2));
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = Vec2)]
    /// # struct Move;
    /// ```
    pub fn mock<A: InputAction>(
        &mut self,
        state: ActionState,
        value: A::Output,
        span: impl Into<MockSpan>,
    ) {
        self.get_or_create_binding::<A>()
            .mock(state, value.into(), span.into());
    }

    /// Clears any active mock for the specified action.
    ///
    /// The action will be resume evaluating real input.
    /// See also [`Self::mock`].
    pub fn clear_mock<A: InputAction>(&mut self) {
        self.get_or_create_binding::<A>().clear_mock();
    }

    fn get_or_create_binding<A: InputAction>(&mut self) -> &mut ActionBinding {
        let type_id = TypeId::of::<A>();
        match self.action_map.entry(type_id) {
            Entry::Occupied(_entry) => self
                .bindings
                .iter_mut()
                .find(|binding| binding.type_id() == type_id)
                .expect("actions and bindings should have matching type IDs"),
            Entry::Vacant(entry) => {
                entry.insert(UntypedAction::new::<A>());
                self.bindings.push(ActionBinding::new::<A>());
                self.bindings.last_mut().unwrap()
            }
        }
    }

    /// Returns bindings for each action in their evaluation order.
    pub fn bindings(&self) -> &[ActionBinding] {
        &self.bindings
    }

    /// Returns the associated bindings for action `A` if exists.
    ///
    /// Use [`Self::bind`] to assign bindings.
    pub fn binding<A: InputAction>(&self) -> Result<&ActionBinding, NoActionError> {
        self.bindings
            .iter()
            .find(|binding| binding.type_id() == TypeId::of::<A>())
            .ok_or(NoActionError::new::<A>())
    }

    /// Returns an iterator over type-erased actions data and their IDs.
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &UntypedAction)> {
        self.action_map.iter().map(|(&id, action)| (id, action))
    }

    /// Returns a mutable iterator over type-erased actions data and their IDs.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TypeId, &mut UntypedAction)> {
        self.action_map.iter_mut().map(|(&id, action)| (id, action))
    }

    /// Returns a type-erased action for the given type ID if it exists.
    pub fn get_mut_by_id(&mut self, type_id: TypeId) -> Option<&mut UntypedAction> {
        self.action_map.get_mut(&type_id)
    }

    /// Returns the associated data for action `A` if it exists.
    ///
    /// Use [`Self::bind`] to associate an action with the context.
    pub fn get<A: InputAction>(&self) -> Result<Action<A>, NoActionError> {
        self.action_map
            .get(&TypeId::of::<A>())
            .map(|action| action.typed())
            .ok_or(NoActionError::new::<A>())
    }

    /// Returns the associated value for action `A` if it exists.
    ///
    /// Helper for [`Self::get`] to the value directly.
    pub fn value<A: InputAction>(&self) -> Result<A::Output, NoActionError> {
        self.get::<A>().map(|action| action.value)
    }

    /// Returns the associated state for action `A` if it exists.
    ///
    /// Helper for [`Self::get`] to the state directly.
    pub fn state<A: InputAction>(&self) -> Result<ActionState, NoActionError> {
        self.get::<A>().map(|action| action.state)
    }

    /// Returns the associated events for action `A` if it exists.
    ///
    /// Helper for [`Self::get`] to the events directly.
    pub fn events<A: InputAction>(&self) -> Result<ActionEvents, NoActionError> {
        self.get::<A>().map(|action| action.events)
    }

    pub(crate) fn update(&mut self, reader: &mut InputReader, time: &InputTime, entity: Entity) {
        if self.sort_required {
            debug!("sorting actions on `{entity}`",);
            self.bindings.sort_by_key(|b| Reverse(b.max_mod_keys()));
            self.sort_required = false;
        }

        reader.set_gamepad(self.gamepad);
        for binding in &mut self.bindings {
            binding.update(reader, &mut self.action_map, time);
        }
    }

    pub(crate) fn trigger(&mut self, commands: &mut Commands, entity: Entity) {
        for binding in &mut self.bindings {
            binding.trigger(commands, &mut self.action_map, entity);
        }
    }

    /// Sets the state for each action to [`ActionState::None`]  and triggers transitions with zero value.
    ///
    /// Resets the input.
    pub(super) fn reset(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &InputTime,
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
        self.sort_required = false;
    }
}

impl<C: InputContext> Default for Actions<C> {
    fn default() -> Self {
        Self {
            actions: UntypedActions::default(),
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct NoActionError {
    action: &'static str,
}

impl NoActionError {
    fn new<A: InputAction>() -> Self {
        Self {
            action: any::type_name::<A>(),
        }
    }
}

impl Display for NoActionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "action `{}` is not defined for this input context",
            self.action,
        )
    }
}

impl Error for NoActionError {}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::{InputAction, InputContext};

    use super::*;

    #[test]
    fn bind() {
        let mut actions = Actions::<Test>::default();
        actions.bind::<TestAction>().to(KeyCode::KeyA);
        actions.bind::<TestAction>().to(KeyCode::KeyB);
        assert_eq!(actions.iter().count(), 1);
        assert_eq!(actions.bindings().len(), 1);

        let binding = actions.binding::<TestAction>().unwrap();
        assert_eq!(binding.inputs().len(), 2);
        assert!(actions.get::<TestAction>().is_ok());
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct TestAction;

    #[derive(InputContext)]
    struct Test;
}
