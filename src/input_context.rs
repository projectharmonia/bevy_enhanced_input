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

use crate::input::input_reader::InputReader;
use context_instance::ContextInstance;

/// An extension trait for [`App`] to register contexts.
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
    mut commands: Commands,
    mut set: ParamSet<(&World, ResMut<ContextInstances>)>,
    time: Res<Time<Virtual>>,
) {
    let mut instances = mem::take(&mut *set.p1());
    instances.rebuild::<C>(set.p0(), &time, &mut commands);
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
    fn add<C: InputContext>(&mut self, world: &World, entity: Entity) {
        debug!("adding `{}` to `{entity}`", any::type_name::<C>());

        if let Some(index) = self.index::<C>() {
            match &mut self.0[index] {
                InstanceGroup::Exclusive { instances, .. } => {
                    let ctx = C::context_instance(world, entity);
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

            let group = InstanceGroup::new::<C>(world, entity);
            self.0.insert(index, group);
        }
    }

    fn rebuild<C: InputContext>(
        &mut self,
        world: &World,
        time: &Time<Virtual>,
        commands: &mut Commands,
    ) {
        if let Some(index) = self.index::<C>() {
            debug!("rebuilding `{}`", any::type_name::<C>());

            match &mut self.0[index] {
                InstanceGroup::Exclusive { instances, .. } => {
                    for (entity, ctx) in instances {
                        ctx.trigger_removed(commands, time, &[*entity]);
                        *ctx = C::context_instance(world, *entity);
                    }
                }
                InstanceGroup::Shared { ctx, entities, .. } => {
                    ctx.trigger_removed(commands, time, entities);

                    // For shared contexts rebuild the instance using the first entity.
                    let entity = *entities
                        .first()
                        .expect("groups should be immediately removed when empty");
                    *ctx = C::context_instance(world, entity);
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
    /// # #[derive(Component)]
    /// # struct Player;
    /// # impl InputContext for Player {
    /// # fn context_instance(_world: &World, _entity: Entity) -> ContextInstance { Default::default() }
    /// # }
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
    fn new<C: InputContext>(world: &World, entity: Entity) -> Self {
        let type_id = TypeId::of::<C>();
        let ctx = C::context_instance(world, entity);
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
///         // component itself from the entity.
///         let player = world.get::<Self>(entity).unwrap();
///
///         let mut ctx = ContextInstance::default();
///
///         ctx.bind::<Move>()
///             .to((GamepadStick::Left, Cardinal::wasd_keys()));
///         ctx.bind::<Jump>()
///             .to((KeyCode::Space, GamepadButton::South));
///
///         ctx
///     }
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

    /// Creates a new instance for the given entity.
    ///
    /// In the implementation you need call [`ContextInstance::bind`]
    /// to associate it with [`InputAction`](input_action::InputAction)s.
    ///
    /// The function is called on each context instantiation
    /// which depends on [`Self::MODE`].
    /// You can also rebuild all contexts by triggering [`RebuildInputContexts`].
    #[must_use]
    fn context_instance(world: &World, entity: Entity) -> ContextInstance;
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
/// This will also reset all actions to [`ActionState::None`](crate::input_context::context_instance::ActionState::None)
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebuildInputContexts;
