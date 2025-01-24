pub mod context_instance;
pub mod events;
pub mod input_action;
pub mod input_bind;
pub mod input_condition;
pub mod input_modifier;
pub mod preset;

use std::{
    any::{self, TypeId},
    cmp::Reverse,
    mem,
};

use bevy::prelude::*;

use crate::input::input_reader::{InputReader, ResetInput};
use context_instance::ContextInstance;

/// An extension trait for [`App`] to register contexts.
///
/// # Examples
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// let mut app = App::new();
/// app.add_input_context::<Player>();
/// # #[derive(Component)]
/// # struct Player;
/// # impl InputContext for Player {
/// # fn context_instance(_world: &World, _entity: Entity) -> ContextInstance { Default::default() }
/// # }
/// ```
pub trait ContextAppExt {
    /// Registers an input context.
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self;
}

impl ContextAppExt for App {
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self {
        debug!("registering context `{}`", any::type_name::<C>());

        self.add_observer(add_instance::<C>)
            .add_observer(rebuild_instance::<C>)
            .add_observer(remove_instance::<C>);

        self
    }
}

fn add_instance<C: InputContext>(
    trigger: Trigger<OnAdd, C>,
    mut set: ParamSet<(&World, ResMut<ContextInstances>)>,
) {
    // We need to borrow both the world and contexts,
    // but we can't use `resource_scope` because observers
    // don't provide mutable access to the world.
    // So we just move it from the resource and put it back.
    let mut instances = mem::take(&mut *set.p1());
    instances.add::<C>(set.p0(), trigger.entity());
    *set.p1() = instances;
}

fn rebuild_instance<C: InputContext>(
    _trigger: Trigger<RebuildInputContexts>,
    mut set: ParamSet<(&World, ResMut<ContextInstances>, ResMut<ResetInput>)>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
) {
    let mut instances = mem::take(&mut *set.p1());
    let mut reset_input = mem::take(&mut *set.p2());
    instances.rebuild::<C>(set.p0(), &mut commands, &mut reset_input, &time);
    *set.p1() = instances;
    *set.p2() = reset_input;
}

fn remove_instance<C: InputContext>(
    trigger: Trigger<OnRemove, C>,
    mut commands: Commands,
    mut reset_input: ResMut<ResetInput>,
    mut instances: ResMut<ContextInstances>,
    time: Res<Time<Virtual>>,
) {
    instances.remove::<C>(&mut commands, &mut reset_input, &time, trigger.entity());
}

/// Stores instantiated [`InputContext`]s.
#[derive(Resource, Default)]
pub struct ContextInstances(Vec<InstanceGroup>);

impl ContextInstances {
    fn add<C: InputContext>(&mut self, world: &World, entity: Entity) {
        debug!("adding `{}` to `{entity}`", any::type_name::<C>());

        if let Some(group) = self
            .0
            .iter_mut()
            .find(|group| group.type_id == TypeId::of::<C>())
        {
            let ctx = C::context_instance(world, entity);
            group.instances.push((entity, ctx));
        } else {
            let priority = Reverse(C::PRIORITY);
            let index = self
                .0
                .binary_search_by_key(&priority, |group| Reverse(group.priority))
                .unwrap_or_else(|e| e);

            let group = InstanceGroup::new::<C>(world, entity);
            self.0.insert(index, group);
        }
    }

    fn rebuild<C: InputContext>(
        &mut self,
        world: &World,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
    ) {
        if let Some(group) = self
            .0
            .iter_mut()
            .find(|group| group.type_id == TypeId::of::<C>())
        {
            debug!("rebuilding `{}`", any::type_name::<C>());
            for (entity, ctx) in &mut group.instances {
                ctx.trigger_removed(commands, reset_input, time, *entity);
                *ctx = C::context_instance(world, *entity);
            }
        }
    }

    fn remove<C: InputContext>(
        &mut self,
        commands: &mut Commands,
        reset_input: &mut ResetInput,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        debug!("removing `{}` from `{entity}`", any::type_name::<C>());

        let group_index = self
            .0
            .iter()
            .position(|group| group.type_id == TypeId::of::<C>())
            .expect("context should be instantiated before removal");

        let group = &mut self.0[group_index];
        let entity_index = group
            .instances
            .iter()
            .position(|&(mapped_entity, _)| mapped_entity == entity)
            .expect("entity should be inserted before removal");

        let (_, mut ctx) = group.instances.swap_remove(entity_index);
        ctx.trigger_removed(commands, reset_input, time, entity);

        if group.instances.is_empty() {
            // Remove the group if no entity references it.
            debug!("removing empty `{}`", any::type_name::<C>());
            self.0.remove(group_index);
        }
    }

    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
    ) {
        for group in &mut self.0 {
            for (entity, ctx) in &mut group.instances {
                ctx.update(commands, reader, time, *entity);
            }
        }
    }

    #[deprecated = "use `ContextInstances::get_context` instead"]
    pub fn get<C: InputContext>(&self, instance_entity: Entity) -> Option<&ContextInstance> {
        self.get_context::<C>(instance_entity)
    }

    /// Returns a context instance for an entity, if it exists.
    ///
    /// For panicking version see [`Self::context`].
    pub fn get_context<C: InputContext>(
        &self,
        instance_entity: Entity,
    ) -> Option<&ContextInstance> {
        let group = self
            .0
            .iter()
            .find(|group| group.type_id == TypeId::of::<C>())?;

        group.instances.iter().find_map(|(entity, ctx)| {
            if *entity == instance_entity {
                Some(ctx)
            } else {
                None
            }
        })
    }

    /// Returns a context instance for an entity.
    ///
    /// For a more ergonomic API, it's recommended to react on [`events`].
    /// within observers.
    ///
    /// The complexity is `O(n+m)`, where `n` is the number of contexts and `m` is the number of entities,
    /// since the storage is optimized for iteration. However, there are usually only a few contexts that are instantiated.
    ///
    /// For non-panicking version see [`Self::get_context`].
    ///
    /// # Panics
    ///
    /// Panics if `C` is not registered as an input context or the entity doesn't have this component.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// fn observer(trigger: Trigger<Attacked>, instances: Res<ContextInstances>) {
    ///     let ctx = instances.context::<Player>(trigger.entity());
    ///     let action = ctx.action::<Dodge>();
    ///     if action.events().contains(ActionEvents::FIRED) {
    ///         // ..
    ///     }
    /// }
    /// # #[derive(Event)]
    /// # struct Attacked;
    /// # #[derive(Component)]
    /// # struct Player;
    /// # impl InputContext for Player {
    /// # fn context_instance(_world: &World, _entity: Entity) -> ContextInstance { Default::default() }
    /// # }
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Dodge;
    /// ```
    pub fn context<C: InputContext>(&self, instance_entity: Entity) -> &ContextInstance {
        self.get_context::<C>(instance_entity).unwrap_or_else(|| {
            panic!(
                "entity `{instance_entity}` should have component `{}` registered as input context",
                any::type_name::<C>()
            )
        })
    }
}

/// Instances of [`InputContext`] for the same type.
struct InstanceGroup {
    type_id: TypeId,
    priority: isize,
    instances: Vec<(Entity, ContextInstance)>,
}

impl InstanceGroup {
    #[must_use]
    fn new<C: InputContext>(world: &World, entity: Entity) -> Self {
        let type_id = TypeId::of::<C>();
        let ctx = C::context_instance(world, entity);
        Self {
            type_id,
            priority: C::PRIORITY,
            instances: vec![(entity, ctx)],
        }
    }
}

/// Contexts are components that associate entities with [`InputAction`](input_action::InputAction)s.
///
/// Inserting this component associates [`ContextInstance`] for this
/// entity in a resource.
///
/// Removing deactivates [`ContextInstance`] for the entity and trigger
/// transitions for all actions to [`ActionState::None`](crate::input_context::context_instance::ActionState::None).
///
/// Each context should be registered using [`ContextAppExt::add_input_context`].
///
/// # Examples
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(Component)]
/// struct Player;
///
/// impl InputContext for Player {
///     fn context_instance(world: &World, entity: Entity) -> ContextInstance {
///         // You can use world to access the necessary data.
///         let settings = world.resource::<AppSettings>();
///
///         // To can also access the context
///         // component itself (or other components) from the entity.
///         let _player = world.get::<Self>(entity).unwrap();
///
///         let mut ctx = ContextInstance::default();
///
///         // When you change your bindings, you can trigger `RebuildInputContexts`
///         // to call this function again to rebuild the context with the the updated bindings.
///         ctx.bind::<Jump>()
///             .to((settings.keyboard.jump, GamepadButton::South));
///
///         ctx
///     }
/// }
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
pub trait InputContext: Component {
    /// Determines the evaluation order of [`ContextInstance`]s produced
    /// by this component.
    ///
    /// Ordering is global.
    /// Contexts with a higher priority evaluated first.
    const PRIORITY: isize = 0;

    /// Creates a new instance for the given entity.
    ///
    /// In the implementation you need call [`ContextInstance::bind`]
    /// to associate it with [`InputAction`](input_action::InputAction)s.
    ///
    /// The function is called on each context instantiation.
    /// You can also rebuild all contexts by triggering [`RebuildInputContexts`].
    #[must_use]
    fn context_instance(world: &World, entity: Entity) -> ContextInstance;
}

/// A trigger that causes the reconstruction of all active context maps.
///
/// Use it when you change your application settings and want to reload the mappings.
///
/// This will also reset all actions to [`ActionState::None`](crate::input_context::context_instance::ActionState::None)
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebuildInputContexts;
