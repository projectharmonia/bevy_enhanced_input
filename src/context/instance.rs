use core::{
    any::{self, TypeId},
    cmp::Reverse,
    marker::PhantomData,
};

use bevy::{
    ecs::{
        schedule::ScheduleLabel,
        world::{FilteredEntityMut, FilteredEntityRef},
    },
    prelude::*,
};

use crate::prelude::*;

/// Stores instantiated [`Actions`] for a schedule `S`.
///
/// Used to iterate over them in a defined order and operate in a type-erased manner.
#[derive(Resource, Default, Deref)]
pub(crate) struct ContextInstances<S: ScheduleLabel> {
    #[deref]
    instances: Vec<ContextInstance>,
    marker: PhantomData<S>,
}

impl<S: ScheduleLabel> ContextInstances<S> {
    pub(super) fn add<C: InputContext>(&mut self, entity: Entity) {
        let instance = ContextInstance::new::<C>(entity);
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

    pub(super) fn remove<C: InputContext>(&mut self, entity: Entity) {
        let index = self
            .iter()
            .position(|inst| inst.entity == entity && inst.type_id == TypeId::of::<C>())
            .expect("input entry should be created before removal");
        self.instances.remove(index);
    }
}

/// Meta information for [`Actions`] on an entity.
pub(crate) struct ContextInstance {
    pub(super) entity: Entity,
    pub(super) name: &'static str,
    type_id: TypeId,
    priority: usize,
    actions: for<'a> fn(&Self, &'a FilteredEntityRef) -> Option<&'a [Entity]>,
    actions_mut: for<'a> fn(&Self, &'a mut FilteredEntityMut) -> Option<&'a mut [Entity]>,
}

impl ContextInstance {
    fn new<C: InputContext>(entity: Entity) -> Self {
        Self {
            entity,
            name: any::type_name::<C>(),
            type_id: TypeId::of::<C>(),
            priority: C::PRIORITY,
            actions: Self::actions_typed::<C>,
            actions_mut: Self::actions_mut_typed::<C>,
        }
    }

    pub(super) fn actions<'a>(&self, contexts: &'a FilteredEntityRef) -> Option<&'a [Entity]> {
        (self.actions)(self, contexts)
    }

    pub(super) fn actions_mut<'a>(
        &self,
        contexts: &'a mut FilteredEntityMut,
    ) -> Option<&'a mut [Entity]> {
        (self.actions_mut)(self, contexts)
    }

    fn actions_typed<'a, C: InputContext>(
        &self,
        entity: &'a FilteredEntityRef,
    ) -> Option<&'a [Entity]> {
        entity.get::<Actions<C>>().map(|actions| &***actions)
    }

    fn actions_mut_typed<'a, C: InputContext>(
        &self,
        entity: &'a mut FilteredEntityMut,
    ) -> Option<&'a mut [Entity]> {
        entity
            .get_mut::<Actions<C>>()
            .map(|actions| &mut **actions.into_inner().collection_mut_risky())
    }
}
