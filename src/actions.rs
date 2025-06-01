use alloc::vec::Vec;
use core::{
    any::{self, TypeId},
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

use bevy::{ecs::schedule::ScheduleLabel, platform::collections::hash_map::Entry, prelude::*};

use crate::{
    action_map::ActionMap,
    input_reader::{InputReader, ResetInput},
    prelude::*,
};

/// Component that stores actions with their bindings for a specific [`InputContext`].
///
/// Bindings represented by [`ActionBinding`] and can be added to specific action using [`Self::bind`].
/// Data for each bound action is stored inside [`ActionMap`].
///
/// Actions are evaluated and trigger [`events`](crate::events) only when this component exists on an entity.
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

    /// Returns the associated bindings for action `A` if exists.
    ///
    /// Use [`Self::bind`] to assign bindings.
    pub fn binding<A: InputAction>(&self) -> Result<&ActionBinding, NoActionError<C, A>> {
        self.bindings
            .iter()
            .find(|binding| binding.type_id() == TypeId::of::<A>())
            .ok_or(NoActionError::default())
    }

    /// Returns the associated data for action `A` if it exists.
    ///
    /// Use [`Self::bind`] to associate an action with the context.
    pub fn get<A: InputAction>(&self) -> Result<&Action, NoActionError<C, A>> {
        self.action_map
            .action::<A>()
            .ok_or(NoActionError::default())
    }

    /// Returns the associated value for action `A` if it exists.
    ///
    /// Helper for [`Self::get`] to the value directly.
    pub fn value<A: InputAction>(&self) -> Result<ActionValue, NoActionError<C, A>> {
        self.get::<A>().map(|action| action.value())
    }

    /// Returns the associated state for action `A` if it exists.
    ///
    /// Helper for [`Self::get`] to the state directly.
    pub fn state<A: InputAction>(&self) -> Result<ActionState, NoActionError<C, A>> {
        self.get::<A>().map(|action| action.state())
    }

    /// Returns the associated events for action `A` if it exists.
    ///
    /// Helper for [`Self::get`] to the events directly.
    pub fn events<A: InputAction>(&self) -> Result<ActionEvents, NoActionError<C, A>> {
        self.get::<A>().map(|action| action.events())
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

pub struct NoActionError<C: InputContext, A: InputAction> {
    context: PhantomData<C>,
    action: PhantomData<A>,
}

impl<C: InputContext, A: InputAction> Default for NoActionError<C, A> {
    fn default() -> Self {
        Self {
            context: PhantomData,
            action: PhantomData,
        }
    }
}

impl<C: InputContext, A: InputAction> Debug for NoActionError<C, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoActionError")
            .field("context", &self.context)
            .field("action", &self.action)
            .finish()
    }
}

impl<C: InputContext, A: InputAction> Display for NoActionError<C, A> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "action `{}` is not present in context `{}`",
            any::type_name::<A>(),
            any::type_name::<C>()
        )
    }
}

impl<C: InputContext, A: InputAction> Error for NoActionError<C, A> {}

/// Marker for a gameplay-related input context that a player can be in.
///
/// Used to differentiate [`Actions`] components and configure how associated actions will be evaluated.
///
/// All structs that implement this trait need to be registered
/// using [`InputContextAppExt::add_input_context`].
///
/// # Examples
///
/// To implement the trait you can use the [`InputContext`]
/// derive to reduce boilerplate:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(InputContext)]
/// struct Player;
/// ```
///
/// Optionally you can pass `priority` and/or `schedule`:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(InputContext)]
/// #[input_context(priority = 1, schedule = FixedPreUpdate)]
/// struct Player;
/// ```
///
/// All parameters match corresponding data in the trait.
pub trait InputContext: Send + Sync + 'static {
    /// Schedule in which the context will be evaluated.
    ///
    /// Associated type defaults are not stabilized in Rust yet,
    /// but the macro uses [`PreUpdate`] by default.
    ///
    /// Set this to [`FixedPreUpdate`] if game logic relies on actions from this context
    /// in [`FixedUpdate`]. For example, if [`FixedMain`](bevy::app::FixedMain) runs twice
    /// in a single frame and an action triggers, you will get [`Started`]
    /// and [`Fired`] on the first run and only [`Fired`] on the second run.
    type Schedule: ScheduleLabel + Default;

    /// Determines the evaluation order of [`Actions<Self>`].
    ///
    /// Used to control how contexts are layered since some [`InputAction`]s may consume inputs.
    ///
    /// Ordering is global. Contexts with a higher priority are evaluated first.
    const PRIORITY: usize = 0;
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::{InputAction, InputContext};

    use super::*;

    #[test]
    fn bind() {
        let mut actions = Actions::<Test>::default();
        actions.bind::<TestAction>().to(KeyCode::KeyA);
        actions.bind::<TestAction>().to(KeyCode::KeyB);
        assert_eq!(actions.bindings.len(), 1);

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
