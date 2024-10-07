pub mod context_map;
pub mod input_action;
pub mod input_condition;
pub mod input_modifier;
pub mod trigger_tracker;

use std::{
    any::{self, TypeId},
    cmp::Reverse,
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

        self.observe(on_context_add::<C>)
            .observe(on_context_remove::<C>);

        self
    }
}

fn on_context_add<C: InputContext>(
    trigger: Trigger<OnAdd, C>,
    mut set: ParamSet<(&World, ResMut<InputContexts>)>,
) {
    debug!(
        "adding input context `{}` to `{}`",
        any::type_name::<C>(),
        trigger.entity(),
    );

    let instace = ContextInstance::new::<C>(set.p0(), trigger.entity());
    set.p1().insert(instace);
}

fn on_context_remove<C: InputContext>(
    trigger: Trigger<OnRemove, C>,
    mut commands: Commands,
    mut contexts: ResMut<InputContexts>,
) {
    debug!(
        "removing input context `{}` from `{}`",
        any::type_name::<C>(),
        trigger.entity()
    );

    let instance = contexts.remove::<C>(trigger.entity());
    instance
        .map
        .trigger_removed(&mut commands, trigger.entity());
}

#[derive(Resource, Default, Deref)]
pub(crate) struct InputContexts(Vec<ContextInstance>);

impl InputContexts {
    fn insert(&mut self, instance: ContextInstance) {
        let priority = Reverse(instance.map.priority());
        let index = self
            .binary_search_by_key(&priority, |reg| Reverse(reg.map.priority()))
            .unwrap_or_else(|e| e);
        self.0.insert(index, instance);
    }

    fn remove<C: InputContext>(&mut self, entity: Entity) -> ContextInstance {
        // TODO: Consider storing per entity.
        let index = self
            .iter()
            .position(|instance| instance.entity == entity && instance.type_id == TypeId::of::<C>())
            .unwrap();
        self.0.remove(index)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut ContextInstance> {
        self.0.iter_mut()
    }
}

pub(crate) struct ContextInstance {
    entity: Entity,
    type_id: TypeId,
    map: ContextMap,
}

impl ContextInstance {
    fn new<C: InputContext>(world: &World, entity: Entity) -> Self {
        Self {
            entity,
            type_id: TypeId::of::<C>(),
            map: C::context_map(world, entity),
        }
    }

    pub(crate) fn update(
        &mut self,
        world: &World,
        commands: &mut Commands,
        reader: &mut InputReader,
        delta: f32,
    ) {
        self.map.update(world, commands, reader, self.entity, delta);
    }
}

pub trait InputContext: Component {
    fn context_map(world: &World, entity: Entity) -> ContextMap;
}
