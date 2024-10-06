pub mod context_map;
pub mod input_action;
pub mod input_condition;
pub mod input_modifier;
pub mod trigger_tracker;

use std::any::{self, TypeId};

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

#[derive(Resource, Default, Deref)]
pub(crate) struct InputContexts(Vec<ContextInstance>);

impl InputContexts {
    fn insert(&mut self, instance: ContextInstance) {
        let index = self
            .binary_search_by_key(&instance.map.priority(), |reg| reg.map.priority())
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
    fn new<C: InputContext>(entity: Entity) -> Self {
        Self {
            entity,
            type_id: TypeId::of::<C>(),
            map: C::context_map(),
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
    const PRIORITY: usize = 0;

    fn context_map() -> ContextMap;
}

fn on_context_add<C: InputContext>(
    trigger: Trigger<OnAdd, C>,
    mut contexts: ResMut<InputContexts>,
) {
    debug!(
        "adding input context `{}` to `{}`",
        any::type_name::<C>(),
        trigger.entity(),
    );

    contexts.insert(ContextInstance::new::<C>(trigger.entity()));
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
