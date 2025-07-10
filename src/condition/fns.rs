use core::ptr;

use bevy::{
    ecs::{
        component::{ComponentId, Mutable},
        world::FilteredEntityMut,
    },
    prelude::*,
};

use crate::prelude::*;

pub trait InputConditionAppExt {
    fn add_input_condition<C: InputCondition + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self;
}

impl InputConditionAppExt for App {
    fn add_input_condition<C: InputCondition + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self {
        let id = self.world_mut().register_component::<C>();
        let mut registry = self.world_mut().resource_mut::<ConditionRegistry>();
        registry.0.push(id);

        self.add_observer(register_condition::<C>)
            .add_observer(unregister_condition::<C>)
            .register_required_components::<C, ConditionFns>()
    }
}

fn register_condition<C: InputCondition + Component<Mutability = Mutable>>(
    trigger: Trigger<OnAdd, C>,
    mut conditions: Query<&mut ConditionFns>,
) {
    let mut fns = conditions.get_mut(trigger.target()).unwrap();
    fns.0.push(get_condition::<C>);
}

fn unregister_condition<C: InputCondition + Component<Mutability = Mutable>>(
    trigger: Trigger<OnRemove, C>,
    mut conditions: Query<&mut ConditionFns>,
) {
    let mut fns = conditions.get_mut(trigger.target()).unwrap();
    let index = fns
        .iter()
        .position(|&f| ptr::fn_addr_eq(f, get_condition::<C> as GetConditionFn))
        .unwrap();
    fns.0.remove(index);
}

#[derive(Resource, Deref, Default)]
pub(crate) struct ConditionRegistry(Vec<ComponentId>);

#[derive(Component, Deref, Default)]
pub(crate) struct ConditionFns(Vec<GetConditionFn>);

type GetConditionFn = for<'a> fn(&'a mut FilteredEntityMut) -> &'a mut dyn InputCondition;

fn get_condition<'a, C: InputCondition + Component<Mutability = Mutable>>(
    entity: &'a mut FilteredEntityMut,
) -> &'a mut dyn InputCondition {
    entity.get_mut::<C>().unwrap().into_inner()
}
