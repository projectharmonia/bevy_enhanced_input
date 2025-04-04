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

/// Instance for [`ActionsMarker`].
///
/// Stores [`InputAction`]s and evaluates their [`ActionState`] in the order they are bound.
///
/// Each action can have multiple associated [`Input`]s, any of which can trigger the action.
///
/// Additionally, you can assign [`InputModifier`]s and [`InputCondition`]s at both the action
/// and input levels.
///
/// You can define bindings before the insertion, but it's recommended to create an observer
/// for [`Binding`](crate::action_instances::Binding). To setup bindings, register an observer
/// an obtain this component. This way you can conveniently reload bindings when you settings
/// change using [`RebuildBindings`](crate::action_instances::RebuildBindings).
///
/// Until the component is exists on the entity, actions will be evaluated and trigger [`events`](super::events).
///
/// Action evaluation follows these steps:
///
/// 1. Iterate over each [`ActionValue`] from the associated [`Input`]s:
///    1.1. Apply input-level [`InputModifier`]s.
///    1.2. Evaluate input-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
/// 2. Select all [`ActionValue`]s with the most significant [`ActionState`] and combine based on [`InputAction::ACCUMULATION`].
///    Combined value be converted into [`ActionOutput::DIM`](crate::input_action::ActionOutput::DIM) using [`ActionValue::convert`].
/// 3. Apply action level [`InputModifier`]s.
/// 4. Evaluate action level [`InputCondition`]s, combining their results according to [`InputCondition::kind`].
/// 5. Set the final [`ActionState`] based on the results.
///    Final value be converted into [`InputAction::Output`] using [`ActionValue::convert`].
///
/// [`InputCondition`]: crate::input_condition::InputCondition
/// [`InputCondition::kind`]: crate::input_condition::InputCondition::kind
/// [`InputModifier`]: crate::input_modifier::InputModifier
/// [`Input`]: crate::input::Input
#[derive(Component)]
pub struct Actions<M: ActionsMarker> {
    gamepad: GamepadDevice,
    bindings: Vec<ActionBinding>,
    action_map: ActionMap,
    marker: PhantomData<M>,
}

impl<M: ActionsMarker> Actions<M> {
    /// Associates context with gamepad.
    ///
    /// By default it's [`GamepadDevice::Any`].
    pub fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        self.gamepad = gamepad.into();
    }

    /// Starts binding an action.
    ///
    /// This method can be called multiple times for the same action to extend its mappings.
    pub fn bind<A: InputAction>(&mut self) -> &mut ActionBinding {
        let type_id = TypeId::of::<A>();
        match self.action_map.entry(type_id) {
            Entry::Occupied(_entry) => self
                .bindings
                .iter_mut()
                .find(|action_bind| action_bind.type_id() == type_id)
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

impl<M: ActionsMarker> Default for Actions<M> {
    fn default() -> Self {
        Self {
            gamepad: Default::default(),
            bindings: Default::default(),
            action_map: Default::default(),
            marker: PhantomData,
        }
    }
}

/// Marker for [`Actions`].
///
/// # Examples
///
/// To implement the trait you can use the [`ActionsMarker`](bevy_enhanced_input_macros::ActionsMarker)
/// derive to reduce boilerplate:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(ActionsMarker)]
/// struct Player;
/// ```
///
/// Optionally you can pass `priority`:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(ActionsMarker)]
/// #[actions_marker(priority = 1)]
/// struct Player;
/// ```
pub trait ActionsMarker: Send + Sync + 'static {
    /// Determines the evaluation order of [`Actions<Self>`].
    ///
    /// Ordering is global.
    /// Contexts with a higher priority evaluated first.
    const PRIORITY: usize = 0;
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::{ActionsMarker, InputAction};

    use super::*;

    #[test]
    fn bind() {
        let mut actions = Actions::<Dummy>::default();
        actions.bind::<DummyAction>().to(KeyCode::KeyA);
        actions.bind::<DummyAction>().to(KeyCode::KeyB);
        assert_eq!(actions.bindings.len(), 1);

        let action = actions.binding::<DummyAction>();
        assert_eq!(action.inputs().len(), 2);
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;

    #[derive(ActionsMarker)]
    struct Dummy;
}
