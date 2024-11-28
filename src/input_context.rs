pub mod context_instance;
pub mod events;
pub mod input_action;
pub mod input_condition;
pub mod input_modifier;
pub mod trigger_tracker;

use std::{
    any::{self, TypeId},
    cell::RefCell,
    cmp::Reverse,
    marker::PhantomData,
    mem,
};

use bevy::prelude::*;

use crate::input::input_reader::InputReader;
use context_instance::ContextInstance;

/// An extension trait for [`App`] to register contexts.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// let mut app = App::new();
/// app.add_input_context::<Player>();
/// # #[derive(Component, InputContext)]
/// # #[input_context(instance_system = instance)]
/// # struct Player;
/// # fn instance(In(_): In<Entity>) -> ContextInstance { default() }
/// ```
pub trait ContextAppExt {
    /// Registers an input context.
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self;
}

impl ContextAppExt for App {
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self {
        debug!("registering context `{}`", any::type_name::<C>());

        let mut instance_system = C::instance_system();
        instance_system.initialize(self.world_mut());
        self.insert_non_send_resource(InstanceSystem::<C>::new(instance_system))
            .observe(add_instance::<C>)
            .observe(rebuild_instance::<C>)
            .observe(remove_instance::<C>);

        self
    }
}

fn add_instance<C: InputContext>(
    trigger: Trigger<OnAdd, C>,
    instance_system: NonSend<InstanceSystem<C>>,
    mut set: ParamSet<(&World, ResMut<ContextInstances>)>,
) {
    // We need to borrow both the world and contexts,
    // but we can't use `resource_scope` because observers
    // don't provide mutable access to the world.
    // So we just move it from the resource and put it back.
    let mut instances = mem::take(&mut *set.p1());
    instances.add::<C>(set.p0(), &instance_system, trigger.entity());
    *set.p1() = instances;
}

fn rebuild_instance<C: InputContext>(
    _trigger: Trigger<RebuildInputContexts>,
    mut commands: Commands,
    mut set: ParamSet<(&World, ResMut<ContextInstances>)>,
    instance_system: NonSend<InstanceSystem<C>>,
    time: Res<Time<Virtual>>,
) {
    let mut instances = mem::take(&mut *set.p1());
    instances.rebuild::<C>(set.p0(), &instance_system, &time, &mut commands);
    *set.p1() = instances;
}

fn remove_instance<C: InputContext>(
    trigger: Trigger<OnRemove, C>,
    mut commands: Commands,
    mut instances: ResMut<ContextInstances>,
    time: Res<Time<Virtual>>,
) {
    instances.remove::<C>(&mut commands, &time, trigger.entity());
}

/// Stores instantiated [`InputContext`]s.
#[derive(Resource, Default)]
pub struct ContextInstances(Vec<InstanceGroup>);

impl ContextInstances {
    fn add<C: InputContext>(
        &mut self,
        world: &World,
        instance_system: &InstanceSystem<C>,
        entity: Entity,
    ) {
        debug!("adding `{}` to `{entity}`", any::type_name::<C>());

        if let Some(index) = self.index::<C>() {
            match &mut self.0[index] {
                InstanceGroup::Exclusive { instances, .. } => {
                    let ctx = instance_system.run(world, entity);
                    instances.push((entity, ctx));
                }
                InstanceGroup::Shared { entities, .. } => {
                    entities.push(entity);
                }
            }
        } else {
            let priority = Reverse(C::PRIORITY);
            let index = self
                .0
                .binary_search_by_key(&priority, |group| Reverse(group.priority()))
                .unwrap_or_else(|e| e);

            let group = InstanceGroup::new::<C>(world, instance_system, entity);
            self.0.insert(index, group);
        }
    }

    fn rebuild<C: InputContext>(
        &mut self,
        world: &World,
        instance_system: &InstanceSystem<C>,
        time: &Time<Virtual>,
        commands: &mut Commands,
    ) {
        if let Some(index) = self.index::<C>() {
            debug!("rebuilding `{}`", any::type_name::<C>());

            match &mut self.0[index] {
                InstanceGroup::Exclusive { instances, .. } => {
                    for (entity, ctx) in instances {
                        ctx.trigger_removed(commands, time, &[*entity]);
                        *ctx = instance_system.run(world, *entity);
                    }
                }
                InstanceGroup::Shared { ctx, entities, .. } => {
                    ctx.trigger_removed(commands, time, entities);

                    // For shared contexts rebuild the instance using the first entity.
                    let entity = *entities
                        .first()
                        .expect("groups should be immediately removed when empty");
                    *ctx = instance_system.run(world, entity);
                }
            }
        }
    }

    fn remove<C: InputContext>(
        &mut self,
        commands: &mut Commands,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        debug!("removing `{}` from `{entity}`", any::type_name::<C>());

        let group_index = self
            .index::<C>()
            .expect("context should be instantiated before removal");

        let empty = match &mut self.0[group_index] {
            InstanceGroup::Exclusive { instances, .. } => {
                let entity_index = instances
                    .iter()
                    .position(|&(mapped_entity, _)| mapped_entity == entity)
                    .expect("entity should be inserted before removal");

                let (_, ctx) = instances.swap_remove(entity_index);
                ctx.trigger_removed(commands, time, &[entity]);

                instances.is_empty()
            }
            InstanceGroup::Shared {
                entities,
                ctx: instance,
                ..
            } => {
                let entity_index = entities
                    .iter()
                    .position(|&mapped_entity| mapped_entity == entity)
                    .expect("entity should be inserted before removal");

                entities.swap_remove(entity_index);
                instance.trigger_removed(commands, time, &[entity]);

                entities.is_empty()
            }
        };

        if empty {
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
            match group {
                InstanceGroup::Exclusive { instances, .. } => {
                    for (entity, ctx) in instances {
                        ctx.update(commands, reader, time, &[*entity]);
                    }
                }
                InstanceGroup::Shared { entities, ctx, .. } => {
                    ctx.update(commands, reader, time, entities);
                }
            }
        }
    }

    /// Returns a context instance for an entity, if it exists.
    ///
    /// For a more ergonomic API, it's recommended to react on [`events`].
    /// within observers.
    ///
    /// The complexity is `O(n+m)`, where `n` is the number of contexts and `m` is the number of entities,
    /// since the storage is optimized for iteration. However, there are usually only a few contexts that are instantiated.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// fn observer(trigger: Trigger<Attacked>, instances: Res<ContextInstances>) {
    ///     let ctx = instances.get::<Player>(trigger.entity()).unwrap();
    ///     let action = ctx.action::<Dodge>().unwrap();
    ///     if action.events().contains(ActionEvents::FIRED) {
    ///         // ..
    ///     }
    /// }
    /// # #[derive(Event)]
    /// # struct Attacked;
    /// # #[derive(Component, InputContext)]
    /// # #[input_context(instance_system = instance)]
    /// # struct Player;
    /// # fn instance(In(_): In<Entity>) -> ContextInstance { default() }
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Dodge;
    /// ```
    pub fn get<C: InputContext>(&self, instance_entity: Entity) -> Option<&ContextInstance> {
        let index = self.index::<C>()?;
        match &self.0[index] {
            InstanceGroup::Exclusive { instances, .. } => {
                instances.iter().find_map(|(entity, ctx)| {
                    if *entity == instance_entity {
                        Some(ctx)
                    } else {
                        None
                    }
                })
            }
            InstanceGroup::Shared { entities, ctx, .. } => {
                entities.contains(&instance_entity).then_some(ctx)
            }
        }
    }

    fn index<C: InputContext>(&self) -> Option<usize> {
        self.0
            .iter()
            .position(|group| group.type_id() == TypeId::of::<C>())
    }
}

/// Instances of [`InputContext`] for the same type based on [`InputContext::MODE`].
enum InstanceGroup {
    Exclusive {
        type_id: TypeId,
        priority: isize,
        instances: Vec<(Entity, ContextInstance)>,
    },
    Shared {
        type_id: TypeId,
        priority: isize,
        entities: Vec<Entity>,
        ctx: ContextInstance,
    },
}

impl InstanceGroup {
    #[must_use]
    fn new<C: InputContext>(
        world: &World,
        instance_system: &InstanceSystem<C>,
        entity: Entity,
    ) -> Self {
        let type_id = TypeId::of::<C>();
        let ctx = instance_system.run(world, entity);
        match C::MODE {
            ContextMode::Exclusive => Self::Exclusive {
                type_id,
                priority: C::PRIORITY,
                instances: vec![(entity, ctx)],
            },
            ContextMode::Shared => Self::Shared {
                type_id,
                priority: C::PRIORITY,
                entities: vec![entity],
                ctx,
            },
        }
    }

    fn priority(&self) -> isize {
        match *self {
            InstanceGroup::Exclusive { priority, .. } => priority,
            InstanceGroup::Shared { priority, .. } => priority,
        }
    }

    fn type_id(&self) -> TypeId {
        match *self {
            InstanceGroup::Exclusive { type_id, .. } => type_id,
            InstanceGroup::Shared { type_id, .. } => type_id,
        }
    }
}

struct InstanceSystem<C> {
    /// System state which can create a [`ContextInstace`].
    ///
    /// This needs to be boxed because we can't name the output type of
    /// [`InputContext::instance_system`].
    ///
    /// We can't make it an associated type either, because `InputContext`
    /// implementors would have to be able to name their system type, which is
    /// practically impossible.
    system: RefCell<Box<dyn ReadOnlySystem<In = Entity, Out = ContextInstance>>>,

    /// Allows monomorphizing this resource for different [`InputContext`]s.
    ///
    /// Otherwise, all `C`s would map to the same resource instance, and would
    /// use the same `system`.
    phantom: PhantomData<C>,
}

impl<C> InstanceSystem<C> {
    fn new(system: impl ReadOnlySystem<In = Entity, Out = ContextInstance>) -> Self {
        Self {
            system: RefCell::new(Box::new(system)),
            phantom: PhantomData,
        }
    }

    fn run(&self, world: &World, entity: Entity) -> ContextInstance {
        self.system.borrow_mut().run_readonly(entity, world)
    }
}

/// Contexts are components that associate entities with [`InputAction`](input_action::InputAction)s.
///
/// Inserting this component associates [`ContextInstance`] for this
/// entity in a resource.
///
/// Removing deactivates [`ContextInstance`] for the entity and trigger
/// transitions for all actions to [`ActionState::None`](input_action::ActionState::None).
///
/// Each context should be registered using [`ContextAppExt::add_input_context`].
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(Component, InputContext)]
/// #[input_context(
///     // Specify the path to the system which creates your `ContextInstance`.
///     instance_system = player_context_instance,
///     // You can also specify extra parameters, like `mode`.
///     mode = Exclusive,
/// )]
/// struct Player;
///
/// fn player_context_instance(
///     In(entity): In<Entity>,
///     // You can use system params to access the data you need.
///     settings: Res<AppSettings>,
///     players: Query<&Player>,
/// ) -> ContextInstance {
///     // You can also access the context
///     // component itself from the entity.
///     let player = players.get(entity).unwrap();
///
///     let mut ctx = ContextInstance::default();
///
///     ctx.bind::<Move>()
///         .with_wasd()
///         .with_stick(GamepadStick::Left);
///
///     ctx.bind::<Jump>()
///         .with(KeyCode::Space)
///         .with(GamepadButtonType::South);
///
///     ctx
/// }
/// # #[derive(Debug, InputAction)]
/// # #[input_action(output = Vec2)]
/// # struct Move;
/// # #[derive(Debug, InputAction)]
/// # #[input_action(output = bool)]
/// # struct Jump;
/// # #[derive(Resource)]
/// # struct AppSettings;
/// ```
pub trait InputContext: Component {
    /// Configures how context will be instantiated.
    const MODE: ContextMode = ContextMode::Exclusive;

    /// Determines the evaluation order of [`ContextInstance`]s produced
    /// by this component.
    ///
    /// Ordering is global.
    /// Contexts with a higher priority evaluated first.
    const PRIORITY: isize = 0;

    /// System which creates a new instance for the given entity.
    ///
    /// In the implementation you need to call [`ContextInstance::bind`]
    /// to associate it with [`InputAction`](input_action::InputAction)s.
    ///
    /// This function is called on [`add_input_context`](ContextAppExt::add_input_context),
    /// and the resulting system is cached. That cached system is called on each
    /// context instantiation, which depends on [`InputContext::MODE`].
    ///
    /// You can also rebuild all contexts by triggering [`RebuildInputContexts`].
    #[must_use]
    fn instance_system() -> impl ReadOnlySystem<In = Entity, Out = ContextInstance>;
}

/// Configures how instances of [`InputContext`] will be managed.
#[derive(Default, Debug)]
pub enum ContextMode {
    /// Instantiate a new context for each entity.
    ///
    /// The context will be created and removed with the component.
    ///
    /// With this mode you can assign different mappings using the same context type.
    ///
    /// Useful for local multiplayer scenarios where each player has different input mappings.
    #[default]
    Exclusive,
    /// Share a single context instance among all entities.
    ///
    /// The context will be created once for the first insertion reused for all other entities
    /// with the same component. It will be removed once no context components of this type exist.
    ///
    /// Useful for games where multiple entities are controlled with the same input.
    Shared,
}

/// A trigger that causes the reconstruction of all active context maps.
///
/// Use it when you change your application settings and want to reload the mappings.
///
/// This will also reset all actions to [`ActionState::None`](crate::input_context::input_action::ActionState::None)
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebuildInputContexts;
