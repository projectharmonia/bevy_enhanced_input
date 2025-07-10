use alloc::slice;
use core::{iter::Copied, marker::PhantomData};

use bevy::{
    ecs::relationship::{RelatedSpawner, RelatedSpawnerCommands},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::context::InputContext;

/// Context entity associated with this action entity.
#[derive(Component, Deref, Reflect, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[relationship(relationship_target = Actions<C>)]
pub struct ActionOf<C: InputContext> {
    #[deref]
    #[relationship]
    entity: Entity,
    marker: PhantomData<C>,
}

impl<C: InputContext> ActionOf<C> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            marker: PhantomData,
        }
    }
}

/// Action entities associated with this context entity.
#[derive(Component, Deref, Reflect, Debug, Default, PartialEq, Eq)]
#[relationship_target(relationship = ActionOf<C>, linked_spawn)]
pub struct Actions<C: InputContext> {
    #[deref]
    #[relationship]
    entities: Vec<Entity>,
    marker: PhantomData<C>,
}

impl<'a, C: InputContext> IntoIterator for &'a Actions<C> {
    type Item = Entity;
    type IntoIter = Copied<slice::Iter<'a, Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A type alias over [`RelatedSpawner`] used to spawn action entities containing an [`ActionOf`] relationship.
pub type ActionSpawner<'w, C> = RelatedSpawner<'w, ActionOf<C>>;

/// A type alias over [`RelatedSpawnerCommands`] used to spawn action entities containing an [`ActionOf`] relationship.
pub type ActionSpawnerCommands<'w, C> = RelatedSpawnerCommands<'w, ActionOf<C>>;

#[macro_export]
macro_rules! actions {
    ($context:ty [$($action:expr),*$(,)?]) => {
       $crate::prelude::Actions::<$context>::spawn(($(::bevy::prelude::Spawn($action)),*))
    };
}
