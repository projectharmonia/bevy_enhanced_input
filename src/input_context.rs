pub mod context_map;
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
use context_map::ContextMap;

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
    mut set: ParamSet<(&World, ResMut<InputContexts>)>,
) {
    // We need to borrow both the world and contexts,
    // but we can't use `resource_scope` because observers
    // don't provide mutable access to the world.
    // So we just move it from the resource and put it back.
    let mut contexts = mem::take(&mut *set.p1());
    contexts.add::<C>(set.p0(), trigger.entity());
    *set.p1() = contexts;
}

fn rebuild_instance<C: InputContext>(
    _trigger: Trigger<RebuildInputContexts>,
    mut commands: Commands,
    mut set: ParamSet<(&World, ResMut<InputContexts>)>,
) {
    let mut contexts = mem::take(&mut *set.p1());
    contexts.rebuild::<C>(set.p0(), &mut commands);
    *set.p1() = contexts;
}

fn remove_instance<C: InputContext>(
    trigger: Trigger<OnRemove, C>,
    mut commands: Commands,
    mut contexts: ResMut<InputContexts>,
) {
    contexts.remove::<C>(&mut commands, trigger.entity());
}

#[derive(Resource, Default)]
pub(crate) struct InputContexts(Vec<ContextInstance>);

impl InputContexts {
    fn add<C: InputContext>(&mut self, world: &World, entity: Entity) {
        debug!("adding `{}` to `{entity}`", any::type_name::<C>());

        if let Some(index) = self.index::<C>() {
            match &mut self.0[index] {
                ContextInstance::Exclusive { maps, .. } => {
                    let map = C::context_map(world, entity);
                    maps.push((entity, map));
                }
                ContextInstance::Shared { entities, .. } => {
                    entities.push(entity);
                }
            }
        } else {
            let priority = Reverse(C::PRIORITY);
            let index = self
                .0
                .binary_search_by_key(&priority, |instance| Reverse(instance.priority()))
                .unwrap_or_else(|e| e);

            let instance = ContextInstance::new::<C>(world, entity);
            self.0.insert(index, instance);
        }
    }

    fn rebuild<C: InputContext>(&mut self, world: &World, commands: &mut Commands) {
        if let Some(index) = self.index::<C>() {
            debug!("rebuilding `{}`", any::type_name::<C>());

            match &mut self.0[index] {
                ContextInstance::Exclusive { maps, .. } => {
                    for (entity, map) in maps {
                        map.trigger_removed(commands, &[*entity]);
                        *map = C::context_map(world, *entity);
                    }
                }
                ContextInstance::Shared { map, entities, .. } => {
                    map.trigger_removed(commands, entities);

                    // For shared contexts rebuild the map using the first entity.
                    let entity = *entities
                        .first()
                        .expect("instances should be immediately removed when empty");
                    *map = C::context_map(world, entity);
                }
            }
        }
    }

    fn remove<C: InputContext>(&mut self, commands: &mut Commands, entity: Entity) {
        debug!("removing `{}` from `{entity}`", any::type_name::<C>());

        let context_index = self
            .index::<C>()
            .expect("context should be instantiated before removal");

        let empty = match &mut self.0[context_index] {
            ContextInstance::Exclusive { maps, .. } => {
                let entity_index = maps
                    .iter()
                    .position(|&(mapped_entity, _)| mapped_entity == entity)
                    .expect("entity should be inserted before removal");

                let (_, map) = maps.swap_remove(entity_index);
                map.trigger_removed(commands, &[entity]);

                maps.is_empty()
            }
            ContextInstance::Shared { entities, map, .. } => {
                let entity_index = entities
                    .iter()
                    .position(|&mapped_entity| mapped_entity == entity)
                    .expect("entity should be inserted before removal");

                entities.swap_remove(entity_index);
                map.trigger_removed(commands, &[entity]);

                entities.is_empty()
            }
        };

        if empty {
            // Remove the instance if no entity references it.
            debug!("removing empty `{}`", any::type_name::<C>());
            self.0.remove(context_index);
        }
    }

    pub(crate) fn update(
        &mut self,
        world: &World,
        commands: &mut Commands,
        reader: &mut InputReader,
        delta: f32,
    ) {
        for instance in &mut self.0 {
            match instance {
                ContextInstance::Exclusive { maps, .. } => {
                    for (entity, map) in maps {
                        map.update(world, commands, reader, &[*entity], delta);
                    }
                }
                ContextInstance::Shared { entities, map, .. } => {
                    map.update(world, commands, reader, entities, delta);
                }
            }
        }
    }

    fn index<C: InputContext>(&mut self) -> Option<usize> {
        self.0
            .iter()
            .position(|instance| instance.type_id() == TypeId::of::<C>())
    }
}

enum ContextInstance {
    Exclusive {
        type_id: TypeId,
        priority: usize,
        maps: Vec<(Entity, ContextMap)>,
    },
    Shared {
        type_id: TypeId,
        priority: usize,
        entities: Vec<Entity>,
        map: ContextMap,
    },
}

impl ContextInstance {
    fn new<C: InputContext>(world: &World, entity: Entity) -> Self {
        let type_id = TypeId::of::<C>();
        let map = C::context_map(world, entity);
        match C::KIND {
            ContextKind::Exclusive => Self::Exclusive {
                type_id,
                priority: C::PRIORITY,
                maps: vec![(entity, map)],
            },
            ContextKind::Shared => Self::Shared {
                type_id,
                priority: C::PRIORITY,
                entities: vec![entity],
                map,
            },
        }
    }

    fn priority(&self) -> usize {
        match *self {
            ContextInstance::Exclusive { priority, .. } => priority,
            ContextInstance::Shared { priority, .. } => priority,
        }
    }

    fn type_id(&self) -> TypeId {
        match *self {
            ContextInstance::Exclusive { type_id, .. } => type_id,
            ContextInstance::Shared { type_id, .. } => type_id,
        }
    }
}

pub trait InputContext: Component {
    const KIND: ContextKind = ContextKind::Shared;
    const PRIORITY: usize = 0;

    fn context_map(world: &World, entity: Entity) -> ContextMap;
}

/// Configures how instances for an input context will be managed.
#[derive(Default, Debug)]
pub enum ContextKind {
    /// Store a separate context for each entity.
    ///
    /// Useful for local multiplayer, where each player has different input mappings.
    #[default]
    Exclusive,

    /// Share a single context instance among all entities.
    ///
    /// Useful for games where multiple characters are controlled with the same input.
    Shared,
}

/// A trigger that causes the reconstruction of all active context maps.
///
/// Use it when you change your application settings and want to reload the mappings.
///
/// This will also reset all actions to [`ActionState::None`](crate::action_state::ActionState::None)
/// and trigger the corresponding events.
#[derive(Event)]
pub struct RebuildInputContexts;
