pub mod context_instance;
pub mod input_action;
pub mod input_condition;
pub mod input_modifier;
pub mod trigger_tracker;

use std::{
    any::{self, TypeId},
    cmp::Reverse,
    mem,
};

use bevy::prelude::*;

use crate::input_reader::InputReader;
use context_instance::ContextInstance;

pub trait ContextAppExt {
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self;
}

impl ContextAppExt for App {
    fn add_input_context<C: InputContext>(&mut self) -> &mut Self {
        debug!("registering context `{}`", any::type_name::<C>());

        self.observe(add_instance::<C>)
            .observe(rebuild_instance::<C>)
            .observe(remove_instance::<C>);

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
) {
    let mut instances = mem::take(&mut *set.p1());
    instances.rebuild::<C>(set.p0(), &mut commands);
    *set.p1() = instances;
}

fn remove_instance<C: InputContext>(
    trigger: Trigger<OnRemove, C>,
    mut commands: Commands,
    mut instances: ResMut<ContextInstances>,
) {
    instances.remove::<C>(&mut commands, trigger.entity());
}

#[derive(Resource, Default)]
pub(crate) struct ContextInstances(Vec<InstanceGroup>);

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

    fn rebuild<C: InputContext>(&mut self, world: &World, commands: &mut Commands) {
        if let Some(index) = self.index::<C>() {
            debug!("rebuilding `{}`", any::type_name::<C>());

            match &mut self.0[index] {
                InstanceGroup::Exclusive { instances, .. } => {
                    for (entity, ctx) in instances {
                        ctx.trigger_removed(commands, &[*entity]);
                        *ctx = C::context_instance(world, *entity);
                    }
                }
                InstanceGroup::Shared { ctx, entities, .. } => {
                    ctx.trigger_removed(commands, entities);

                    // For shared contexts rebuild the instance using the first entity.
                    let entity = *entities
                        .first()
                        .expect("groups should be immediately removed when empty");
                    *ctx = C::context_instance(world, entity);
                }
            }
        }
    }

    fn remove<C: InputContext>(&mut self, commands: &mut Commands, entity: Entity) {
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
                ctx.trigger_removed(commands, &[entity]);

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
                instance.trigger_removed(commands, &[entity]);

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
        world: &World,
        commands: &mut Commands,
        reader: &mut InputReader,
        delta: f32,
    ) {
        for group in &mut self.0 {
            match group {
                InstanceGroup::Exclusive { instances, .. } => {
                    for (entity, ctx) in instances {
                        ctx.update(world, commands, reader, &[*entity], delta);
                    }
                }
                InstanceGroup::Shared { entities, ctx, .. } => {
                    ctx.update(world, commands, reader, entities, delta);
                }
            }
        }
    }

    fn index<C: InputContext>(&mut self) -> Option<usize> {
        self.0
            .iter()
            .position(|group| group.type_id() == TypeId::of::<C>())
    }
}

/// Instances of [`InputContext`] for the same type.
enum InstanceGroup {
    Exclusive {
        type_id: TypeId,
        priority: usize,
        instances: Vec<(Entity, ContextInstance)>,
    },
    Shared {
        type_id: TypeId,
        priority: usize,
        entities: Vec<Entity>,
        ctx: ContextInstance,
    },
}

impl InstanceGroup {
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

    fn priority(&self) -> usize {
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

pub trait InputContext: Component {
    const MODE: ContextMode = ContextMode::Exclusive;
    const PRIORITY: usize = 0;

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
/// This will also reset all actions to [`ActionState::None`](crate::input_context::input_action::ActionState::None)
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebuildInputContexts;
