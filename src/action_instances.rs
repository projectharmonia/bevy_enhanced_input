use alloc::vec::Vec;
use core::{
    any::{self, TypeId},
    cmp::Reverse,
    marker::PhantomData,
};

use bevy::{
    ecs::{component::ComponentId, world::FilteredEntityMut},
    prelude::*,
};

use super::{
    Actions,
    actions::ActionsMarker,
    input_reader::{InputReader, ResetInput},
};

/// An extension trait for [`App`] to assign input to components.
pub trait ActionsMarkerAppExt {
    /// Registers `T` an input context marker.
    ///
    /// Necessary to update [`Actions<T>`] components.
    ///
    /// Component removal deactivates [`ActionsMarker`] for the entity and trigger transitions for all actions
    /// to [`ActionState::None`](super::ActionState::None).
    ///
    /// Component re-insertion re-triggers [`Binding`]. Use it if you want to reload bindings.
    /// You can also rebuild all component bindings by triggering [`RebuildBindings`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(EnhancedInputPlugin);
    /// app.add_actions_marker::<Player>()
    ///     .add_observer(player_binding);
    ///
    /// fn player_binding(
    ///     trigger: Trigger<Binding<Player>>,
    ///     settings: Res<AppSettings>,
    ///     mut players: Query<&mut Actions<Player>>,
    /// ) {
    ///     let mut actions = players.get_mut(trigger.entity()).unwrap();
    ///     actions
    ///         .bind::<Jump>()
    ///         .to((settings.keyboard.jump, GamepadButton::South));
    /// }
    ///
    /// #[derive(ActionsMarker)]
    /// struct Player;
    ///
    /// #[derive(Debug, InputAction)]
    /// #[input_action(output = bool)]
    /// struct Jump;
    ///
    /// #[derive(Resource)]
    /// struct AppSettings {
    ///     keyboard: KeyboardSettings,
    /// }
    ///
    /// struct KeyboardSettings {
    ///     jump: KeyCode,
    /// }
    /// ```
    fn add_actions_marker<M: ActionsMarker>(&mut self) -> &mut Self;
}

impl ActionsMarkerAppExt for App {
    fn add_actions_marker<M: ActionsMarker>(&mut self) -> &mut Self {
        debug!("registering context for `{}`", any::type_name::<M>());

        let id = self.world_mut().register_component::<Actions<M>>();
        let mut registry = self.world_mut().resource_mut::<ActionsRegistry>();
        registry.push(id);

        self.add_observer(add_context::<M>)
            .add_observer(remove_context::<M>);

        self
    }
}

/// IDs of registered [`Actions`].
///
/// Used later to configure [`FilteredEntityMut`].
/// Exists only at plugins initialization stage.
#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct ActionsRegistry(Vec<ComponentId>);

fn add_context<M: ActionsMarker>(
    trigger: Trigger<OnInsert, Actions<M>>,
    mut commands: Commands,
    mut instances: ResMut<ActionInstances>,
) {
    instances.add::<M>(&mut commands, trigger.entity());
}

fn remove_context<M: ActionsMarker>(
    trigger: Trigger<OnReplace, Actions<M>>,
    mut commands: Commands,
    mut reset_input: ResMut<ResetInput>,
    mut instances: ResMut<ActionInstances>,
    time: Res<Time<Virtual>>,
    mut actions: Query<&mut Actions<M>>,
) {
    instances.remove::<M>(
        &mut commands,
        &mut reset_input,
        &time,
        &mut actions,
        trigger.entity(),
    );
}

/// Stores instantiated [`Actions`].
///
/// Used to iterate over them in a defined order and operate in a type-erased manner.
#[derive(Resource, Default, Deref)]
pub(super) struct ActionInstances(Vec<ActionsInstance>);

impl ActionInstances {
    pub(super) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        for instance in &mut self.0 {
            instance.update(commands, reader, time, actions);
        }
    }

    pub(super) fn rebuild(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        for instance in &mut self.0 {
            instance.rebuild(commands, reset_input, time, actions);
        }
    }

    fn add<M: ActionsMarker>(&mut self, commands: &mut Commands, entity: Entity) {
        debug!(
            "adding input context for `{}` to `{entity}`",
            any::type_name::<M>(),
        );

        commands.trigger_targets(Binding::<M>::new(), entity);

        let instance = ActionsInstance::new::<M>(entity);
        match self.binary_search_by_key(&Reverse(M::PRIORITY), |inst| Reverse(inst.priority)) {
            Ok(index) => {
                // Insert last to preserve entry creation order.
                let last_priority_index = self
                    .iter()
                    .skip(index + 1)
                    .position(|inst| inst.priority != M::PRIORITY)
                    .unwrap_or_default();
                self.0.insert(index + last_priority_index + 1, instance);
            }
            Err(index) => self.0.insert(index, instance),
        };
    }

    fn remove<M: ActionsMarker>(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        instances: &mut Query<&mut Actions<M>>,
        entity: Entity,
    ) {
        debug!(
            "removing input context for `{}` from `{}`",
            any::type_name::<M>(),
            entity
        );

        let index = self
            .iter()
            .position(|inst| inst.entity == entity && inst.type_id == TypeId::of::<M>())
            .expect("input entry should be created before removal");
        self.0.remove(index);

        let mut instance = instances.get_mut(entity).unwrap();
        instance.reset(commands, reset_input, time, entity);
    }
}

/// Meta information for [`Actions`] on an entity.
pub(super) struct ActionsInstance {
    entity: Entity,
    priority: usize,
    type_id: TypeId,

    // Type-erased functions.
    update: UpdateFn,
    rebuild: RebuildFn,
}

impl ActionsInstance {
    fn new<M: ActionsMarker>(entity: Entity) -> Self {
        Self {
            entity,
            priority: M::PRIORITY,
            type_id: TypeId::of::<M>(),
            // Since the type is not present in the signature, we can store
            // functions for specific type without making the struct generic.
            update: Self::update_typed::<M>,
            rebuild: Self::rebuild_typed::<M>,
        }
    }

    /// Calls [`Self::update_typed`] for `M` that was associated in [`Self::new`].
    fn update(
        &self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        (self.update)(self, commands, reader, time, actions);
    }

    /// Calls [`Self::rebuild_typed`] for `M` that was associated in [`Self::new`].
    fn rebuild(
        &self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        (self.rebuild)(self, commands, reset_input, time, actions);
    }

    fn update_typed<M: ActionsMarker>(
        &self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        trace!(
            "updating bindings for `{}` on `{}`",
            any::type_name::<M>(),
            self.entity
        );

        let mut actions = actions
            .get_mut(self.entity)
            .ok()
            .and_then(FilteredEntityMut::into_mut::<Actions<M>>)
            .expect("deinitialized instances should be previously removed");

        actions.update(commands, reader, time, self.entity);
    }

    fn rebuild_typed<M: ActionsMarker>(
        &self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        debug!(
            "resetting bindings for `{}` on `{}`",
            any::type_name::<M>(),
            self.entity
        );

        let mut actions = actions
            .get_mut(self.entity)
            .ok()
            .and_then(FilteredEntityMut::into_mut::<Actions<M>>)
            .expect("deinitialized instances should be previously removed");

        actions.reset(commands, reset_input, time, self.entity);
        commands.trigger_targets(Binding::<M>::new(), self.entity);
    }
}

type UpdateFn = fn(
    &ActionsInstance,
    &mut Commands,
    &mut InputReader,
    &Time<Virtual>,
    &mut Query<FilteredEntityMut>,
);

type RebuildFn = fn(
    &ActionsInstance,
    &mut Commands,
    &mut ResetInput,
    &Time<Virtual>,
    &mut Query<FilteredEntityMut>,
);

/// Trigger that requests bindings creation of [`Actions`] for an entity.
///
/// Can't be triggered by user. If you want to reload bindings, just re-insert
/// the component or trigger [`RebuildBindings`].
#[derive(Event)]
pub struct Binding<M: ActionsMarker>(PhantomData<M>);

impl<M: ActionsMarker> Binding<M> {
    /// Creates a new instance.
    ///
    /// Not exposed to users to because we need to properly
    /// trigger events on bindings rebuild.
    fn new() -> Self {
        Self(PhantomData)
    }
}

/// A trigger that causes the reconstruction of all active [`Actions`].
///
/// Use it when you change your application settings and want to reload the mappings.
///
/// This will also reset all actions to [`ActionState::None`](super::ActionState::None)
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebuildBindings;
