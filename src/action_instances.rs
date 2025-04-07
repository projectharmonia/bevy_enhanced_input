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

use crate::{
    actions::{Actions, InputContext},
    input_reader::{InputReader, ResetInput},
};

/// An extension trait for [`App`] to assign input to components.
pub trait InputContextAppExt {
    /// Registers type `C` as an input context.
    ///
    /// All structs that implement [`InputContext`] must be registered,
    /// otherwise [`Actions<C>`] won't be evaluated.
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self;
}

impl InputContextAppExt for App {
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self {
        debug!("registering context for `{}`", any::type_name::<C>());

        let id = self.world_mut().register_component::<Actions<C>>();
        let mut registry = self.world_mut().resource_mut::<ActionsRegistry>();
        debug_assert!(
            !registry.contains(&id),
            "context `{}` can't be added more then once",
            any::type_name::<C>()
        );
        registry.push(id);

        self.add_observer(add_context::<C>)
            .add_observer(remove_context::<C>);

        self
    }
}

/// IDs of registered [`Actions`].
///
/// Used later to configure [`FilteredEntityMut`].
/// Exists only at plugins initialization stage.
#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct ActionsRegistry(Vec<ComponentId>);

fn add_context<C: InputContext>(
    trigger: Trigger<OnInsert, Actions<C>>,
    mut commands: Commands,
    mut instances: ResMut<ActionInstances>,
) {
    instances.add::<C>(&mut commands, trigger.entity());
}

fn remove_context<C: InputContext>(
    trigger: Trigger<OnReplace, Actions<C>>,
    mut commands: Commands,
    mut reset_input: ResMut<ResetInput>,
    mut instances: ResMut<ActionInstances>,
    time: Res<Time<Virtual>>,
    mut actions: Query<&mut Actions<C>>,
) {
    instances.remove::<C>(
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
pub(crate) struct ActionInstances(Vec<ActionsInstance>);

impl ActionInstances {
    pub(crate) fn update(
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

    pub(crate) fn rebuild(
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

    fn add<C: InputContext>(&mut self, commands: &mut Commands, entity: Entity) {
        debug!(
            "adding input context `{}` to `{entity}`",
            any::type_name::<C>(),
        );

        commands.trigger_targets(Binding::<C>::new(), entity);

        let instance = ActionsInstance::new::<C>(entity);
        match self.binary_search_by_key(&Reverse(C::PRIORITY), |inst| Reverse(inst.priority)) {
            Ok(index) => {
                // Insert last to preserve entry creation order.
                let last_priority_index = self
                    .iter()
                    .skip(index + 1)
                    .position(|inst| inst.priority != C::PRIORITY)
                    .unwrap_or_default();
                self.0.insert(index + last_priority_index + 1, instance);
            }
            Err(index) => self.0.insert(index, instance),
        };
    }

    fn remove<C: InputContext>(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        instances: &mut Query<&mut Actions<C>>,
        entity: Entity,
    ) {
        debug!(
            "removing input context `{}` from `{}`",
            any::type_name::<C>(),
            entity
        );

        let index = self
            .iter()
            .position(|inst| inst.entity == entity && inst.type_id == TypeId::of::<C>())
            .expect("input entry should be created before removal");
        self.0.remove(index);

        let mut instance = instances.get_mut(entity).unwrap();
        instance.reset(commands, reset_input, time, entity);
    }
}

/// Meta information for [`Actions`] on an entity.
pub(crate) struct ActionsInstance {
    entity: Entity,
    priority: usize,
    type_id: TypeId,

    // Type-erased functions.
    update: UpdateFn,
    rebuild: RebuildFn,
}

impl ActionsInstance {
    fn new<C: InputContext>(entity: Entity) -> Self {
        Self {
            entity,
            priority: C::PRIORITY,
            type_id: TypeId::of::<C>(),
            // Since the type is not present in the signature, we can store
            // functions for specific type without making the struct generic.
            update: Self::update_typed::<C>,
            rebuild: Self::rebuild_typed::<C>,
        }
    }

    /// Calls [`Self::update_typed`] for `C` that was associated in [`Self::new`].
    fn update(
        &self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        (self.update)(self, commands, reader, time, actions);
    }

    /// Calls [`Self::rebuild_typed`] for `C` that was associated in [`Self::new`].
    fn rebuild(
        &self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        (self.rebuild)(self, commands, reset_input, time, actions);
    }

    fn update_typed<C: InputContext>(
        &self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        trace!(
            "updating input context `{}` on `{}`",
            any::type_name::<C>(),
            self.entity
        );

        let mut actions = actions
            .get_mut(self.entity)
            .ok()
            .and_then(FilteredEntityMut::into_mut::<Actions<C>>)
            .expect("deinitialized instances should be previously removed");

        actions.update(commands, reader, time, self.entity);
    }

    fn rebuild_typed<C: InputContext>(
        &self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        debug!(
            "resetting input context `{}` on `{}`",
            any::type_name::<C>(),
            self.entity
        );

        let mut actions = actions
            .get_mut(self.entity)
            .ok()
            .and_then(FilteredEntityMut::into_mut::<Actions<C>>)
            .expect("deinitialized instances should be previously removed");

        actions.reset(commands, reset_input, time, self.entity);
        commands.trigger_targets(Binding::<C>::new(), self.entity);
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
pub struct Binding<C: InputContext>(PhantomData<C>);

impl<C: InputContext> Binding<C> {
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
