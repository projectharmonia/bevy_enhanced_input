use core::ptr;

use bevy::{
    ecs::{
        component::{ComponentId, Mutable},
        world::FilteredEntityMut,
    },
    prelude::*,
};

use crate::prelude::*;

pub trait InputModifierAppExt {
    fn add_input_modifier<M: InputModifier + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self;
}

impl InputModifierAppExt for App {
    fn add_input_modifier<M: InputModifier + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self {
        let id = self.world_mut().register_component::<M>();
        let mut registry = self.world_mut().resource_mut::<ModifierRegistry>();
        registry.0.push(id);

        self.add_observer(register_modifier::<M>)
            .add_observer(unregister_modifier::<M>)
            .register_required_components::<M, ModifierFns>()
    }
}

fn register_modifier<M: InputModifier + Component<Mutability = Mutable>>(
    trigger: Trigger<OnAdd, M>,
    mut modifiers: Query<&mut ModifierFns>,
) {
    let mut fns = modifiers.get_mut(trigger.target()).unwrap();
    fns.0.push(get_modifier::<M>);
}

fn unregister_modifier<M: InputModifier + Component<Mutability = Mutable>>(
    trigger: Trigger<OnRemove, M>,
    mut modifiers: Query<&mut ModifierFns>,
) {
    let mut fns = modifiers.get_mut(trigger.target()).unwrap();
    let index = fns
        .iter()
        .position(|&f| ptr::fn_addr_eq(f, get_modifier::<M> as GetModifierFn))
        .unwrap();
    fns.0.remove(index);
}

#[derive(Resource, Deref, Default)]
pub(crate) struct ModifierRegistry(Vec<ComponentId>);

#[derive(Component, Deref, Default)]
pub(crate) struct ModifierFns(Vec<GetModifierFn>);

type GetModifierFn = for<'a> fn(&'a mut FilteredEntityMut) -> &'a mut dyn InputModifier;

fn get_modifier<'a, C: InputModifier + Component<Mutability = Mutable>>(
    entity: &'a mut FilteredEntityMut,
) -> &'a mut dyn InputModifier {
    entity.get_mut::<C>().unwrap().into_inner()
}
