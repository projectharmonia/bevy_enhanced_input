use alloc::slice;
use core::iter::Copied;

use bevy::{
    ecs::relationship::{RelatedSpawner, RelatedSpawnerCommands},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Action entity associated with this binding entity.
#[derive(Component, Deref, Reflect, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[relationship(relationship_target = Bindings)]
pub struct BindingOf(pub Entity);

/// Binding entities associated with this action entity.
#[derive(Component, Deref, Reflect, Debug, Default, PartialEq, Eq)]
#[relationship_target(relationship = BindingOf, linked_spawn)]
pub struct Bindings(Vec<Entity>);

impl<'a> IntoIterator for &'a Bindings {
    type Item = Entity;
    type IntoIter = Copied<slice::Iter<'a, Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A type alias over [`RelatedSpawner`] used to spawn binding entities containing a [`BindingOf`] relationship.
pub type BindingSpawner<'w> = RelatedSpawner<'w, BindingOf>;

/// A type alias over [`RelatedSpawnerCommands`] used to spawn binding entities containing a [`BindingOf`] relationship.
pub type BindingSpawnerCommands<'w> = RelatedSpawnerCommands<'w, BindingOf>;

#[macro_export]
macro_rules! bindings {
    [ $( ( $first:expr $(, $rest:expr )* ) ),* $(,)? ] => {
        $crate::prelude::Bindings::spawn((
            $( ::bevy::prelude::Spawn(($crate::prelude::Binding::from($first), $($rest),*)) ),*
        ))
    };

    [ $( $binding:expr ),* $(,)? ] => {
        $crate::prelude::Bindings::spawn((
            $( ::bevy::prelude::Spawn($crate::prelude::Binding::from($binding)) ),*
        ))
    };
}
