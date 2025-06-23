pub mod action_binding;
pub mod actions;
pub mod events;
pub mod input_action;
pub mod input_binding;
pub mod input_condition;
pub mod input_modifier;
pub mod preset;
mod trigger_tracker;

use alloc::vec::Vec;
use core::{
    any::{self, TypeId},
    cmp::Reverse,
    marker::PhantomData,
};

use bevy::{
    ecs::{
        component::ComponentId,
        schedule::ScheduleLabel,
        system::{ParamBuilder, QueryParamBuilder},
        world::FilteredEntityMut,
    },
    prelude::*,
};
use log::{debug, trace};

use crate::{
    EnhancedInputSet,
    input_reader::{InputReader, ResetInput},
    prelude::*,
};

/// An extension trait for [`App`] to assign input to components.
pub trait InputContextAppExt {
    /// Registers type `C` as an input context.
    ///
    /// Any struct `C` that implements [`InputContext`] must be registered,
    /// otherwise [`Actions<C>`] won't be evaluated.
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self;
}

impl InputContextAppExt for App {
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self {
        debug!(
            "registering `{}` for `{}`",
            any::type_name::<C>(),
            any::type_name::<C::Schedule>(),
        );

        let id = self.world_mut().register_component::<Actions<C>>();
        let mut registry = self.world_mut().resource_mut::<ContextRegistry>();
        if let Some(contexts) = registry
            .iter_mut()
            .find(|c| c.schedule_id == TypeId::of::<C::Schedule>())
        {
            debug_assert!(
                !contexts.action_ids.contains(&id),
                "context `{}` shouldn't be added more then once",
                any::type_name::<C>()
            );
            contexts.action_ids.push(id);
        } else {
            let mut contexts = ScheduleContexts::new::<C::Schedule>();
            contexts.action_ids.push(id);
            registry.push(contexts);
        }

        self.add_observer(add_context::<C>)
            .add_observer(remove_context::<C>);

        self
    }
}

/// Tracks registered input contexts for each [`InputContext::Schedule`].
///
/// In Bevy, it’s impossible to know which schedule is used inside a system,
/// so we genericize update systems over schedules.
///
/// This resource stores registered contexts per-schedule in a type-erased way
/// to perform the setup after all registrations in [`App::finish`].
///
/// Exists only during the plugin initialization.
#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct ContextRegistry(Vec<ScheduleContexts>);

pub(crate) struct ScheduleContexts {
    /// Schedule ID for which all actions were registered.
    schedule_id: TypeId,

    /// IDs of [`Actions`]
    action_ids: Vec<ComponentId>,

    /// Configures the app for this schedule.
    setup: fn(&Self, &mut App),
}

impl ScheduleContexts {
    /// Creates a new instance for schedule `S`.
    ///
    /// [`Self::setup`] will configure the app for `S`.
    fn new<S: ScheduleLabel + Default>() -> Self {
        Self {
            schedule_id: TypeId::of::<S>(),
            action_ids: Default::default(),
            // Since the type is not present in the function signature, we can store
            // functions for specific type without making the struct generic.
            setup: Self::setup_typed::<S>,
        }
    }

    /// Calls [`Self::setup_typed`] for `S` that was associated in [`Self::new`].
    pub(crate) fn setup(&self, app: &mut App) {
        (self.setup)(self, app);
    }

    /// Configures the app for all contexts registered for schedule `C`.
    pub(crate) fn setup_typed<S: ScheduleLabel + Default>(&self, app: &mut App) {
        let update = (
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder.optional(|builder| {
                    for &id in &self.action_ids {
                        builder.mut_id(id);
                    }
                });
            }),
        )
            .build_state(app.world_mut())
            .build_system(update::<S>);

        let trigger = (
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder.optional(|builder| {
                    for &id in &self.action_ids {
                        builder.mut_id(id);
                    }
                });
            }),
        )
            .build_state(app.world_mut())
            .build_system(trigger::<S>);

        let rebuild = (
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder.optional(|builder| {
                    for &id in &self.action_ids {
                        builder.mut_id(id);
                    }
                });
            }),
        )
            .build_state(app.world_mut())
            .build_any_system(rebuild::<S>);

        app.init_resource::<ActionInstances<S>>()
            .configure_sets(
                S::default(),
                (EnhancedInputSet::Update, EnhancedInputSet::Trigger).chain(),
            )
            .add_observer(rebuild)
            .add_systems(
                S::default(),
                (
                    update.in_set(EnhancedInputSet::Update),
                    trigger.in_set(EnhancedInputSet::Trigger),
                ),
            );
    }
}

fn add_context<C: InputContext>(
    trigger: Trigger<OnInsert, Actions<C>>,
    mut commands: Commands,
    mut instances: ResMut<ActionInstances<C::Schedule>>,
) {
    instances.add::<C>(&mut commands, trigger.target());
}

fn remove_context<C: InputContext>(
    trigger: Trigger<OnReplace, Actions<C>>,
    mut commands: Commands,
    time: InputTime,
    mut reset_input: ResMut<ResetInput>,
    mut instances: ResMut<ActionInstances<C::Schedule>>,
    mut actions: Query<&mut Actions<C>>,
) {
    instances.remove::<C>(
        &mut commands,
        &mut reset_input,
        &time,
        &mut actions,
        trigger.target(),
    );
}

fn update<S: ScheduleLabel>(
    time: InputTime,
    mut reader: InputReader,
    mut instances: ResMut<ActionInstances<S>>,
    mut actions: Query<FilteredEntityMut>,
) {
    reader.update_state();
    instances.update(&mut reader, &time, &mut actions);
}

fn trigger<S: ScheduleLabel>(
    mut commands: Commands,
    mut instances: ResMut<ActionInstances<S>>,
    mut actions: Query<FilteredEntityMut>,
) {
    instances.trigger(&mut commands, &mut actions);
}

fn rebuild<S: ScheduleLabel>(
    _trigger: Trigger<RebindAll>,
    mut commands: Commands,
    time: InputTime,
    mut reset_input: ResMut<ResetInput>,
    mut instances: ResMut<ActionInstances<S>>,
    mut actions: Query<FilteredEntityMut>,
) {
    instances.rebuild(&mut commands, &mut reset_input, &time, &mut actions);
}

/// Stores instantiated [`Actions`] for a schedule `S`.
///
/// Used to iterate over them in a defined order and operate in a type-erased manner.
#[derive(Resource, Default, Deref)]
pub(crate) struct ActionInstances<S: ScheduleLabel> {
    #[deref]
    instances: Vec<ActionsInstance>,
    marker: PhantomData<S>,
}

impl<S: ScheduleLabel> ActionInstances<S> {
    pub(crate) fn update(
        &mut self,
        reader: &mut InputReader,
        time: &InputTime,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        for instance in &mut self.instances {
            instance.update(reader, time, actions);
        }
    }

    pub(crate) fn trigger(
        &mut self,
        commands: &mut Commands,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        for instance in &mut self.instances {
            instance.trigger(commands, actions);
        }
    }

    pub(crate) fn rebuild(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &InputTime,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        for instance in &mut self.instances {
            instance.rebuild(commands, reset_input, time, actions);
        }
    }

    fn add<C: InputContext>(&mut self, commands: &mut Commands, entity: Entity) {
        debug!(
            "adding input context `{}` to `{entity}`",
            any::type_name::<C>(),
        );

        commands.trigger_targets(Bind::<C>::new(), entity);

        let instance = ActionsInstance::new::<C>(entity);
        match self.binary_search_by_key(&Reverse(C::PRIORITY), |inst| Reverse(inst.priority)) {
            Ok(index) => {
                // Insert last to preserve entry creation order.
                let last_priority_index = self
                    .iter()
                    .skip(index + 1)
                    .position(|inst| inst.priority != C::PRIORITY)
                    .unwrap_or_default();
                self.instances
                    .insert(index + last_priority_index + 1, instance);
            }
            Err(index) => self.instances.insert(index, instance),
        };
    }

    fn remove<C: InputContext>(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &InputTime,
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
        self.instances.remove(index);

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
    trigger: TriggerFn,
    rebuild: RebuildFn,
}

impl ActionsInstance {
    fn new<C: InputContext>(entity: Entity) -> Self {
        Self {
            entity,
            priority: C::PRIORITY,
            type_id: TypeId::of::<C>(),
            update: Self::update_typed::<C>,
            trigger: Self::trigger_typed::<C>,
            rebuild: Self::rebuild_typed::<C>,
        }
    }

    /// Calls [`Self::update_typed`] for `C` that was associated in [`Self::new`].
    fn update(
        &self,
        reader: &mut InputReader,
        time: &InputTime,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        (self.update)(self, reader, time, actions);
    }

    /// Calls [`Self::trigger_typed`] for `C` that was associated in [`Self::new`].
    fn trigger(&self, commands: &mut Commands, actions: &mut Query<FilteredEntityMut>) {
        (self.trigger)(self, commands, actions);
    }

    /// Calls [`Self::rebuild_typed`] for `C` that was associated in [`Self::new`].
    fn rebuild(
        &self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &InputTime,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        (self.rebuild)(self, commands, reset_input, time, actions);
    }

    fn update_typed<C: InputContext>(
        &self,
        reader: &mut InputReader,
        time: &InputTime,
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

        actions.update(reader, time, self.entity);
    }

    fn trigger_typed<C: InputContext>(
        &self,
        commands: &mut Commands,
        actions: &mut Query<FilteredEntityMut>,
    ) {
        trace!(
            "triggering input context `{}` on `{}`",
            any::type_name::<C>(),
            self.entity
        );

        let mut actions = actions
            .get_mut(self.entity)
            .ok()
            .and_then(FilteredEntityMut::into_mut::<Actions<C>>)
            .expect("deinitialized instances should be previously removed");

        actions.trigger(commands, self.entity);
    }

    fn rebuild_typed<C: InputContext>(
        &self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &InputTime,
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
        commands.trigger_targets(Bind::<C>::new(), self.entity);
    }
}

type UpdateFn = fn(&ActionsInstance, &mut InputReader, &InputTime, &mut Query<FilteredEntityMut>);

type TriggerFn = fn(&ActionsInstance, &mut Commands, &mut Query<FilteredEntityMut>);

type RebuildFn =
    fn(&ActionsInstance, &mut Commands, &mut ResetInput, &InputTime, &mut Query<FilteredEntityMut>);

/// Trigger that requests bindings creation of [`Actions`] for an entity.
///
/// Can't be triggered by user. If you want to reload bindings, just re-insert
/// the component or trigger [`RebindAll`].
#[derive(Event)]
pub struct Bind<C: InputContext>(PhantomData<C>);

impl<C: InputContext> Bind<C> {
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
/// This will also reset all actions to [`ActionState::None`]
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebindAll;

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
