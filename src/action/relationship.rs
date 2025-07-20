use alloc::slice;
use core::{iter::Copied, marker::PhantomData};

use bevy::{
    ecs::relationship::{RelatedSpawner, RelatedSpawnerCommands},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Entity with context `C` associated with this action entity.
///
/// See also the [`actions!`] macro for conveniently spawning associated actions.
#[derive(Component, Deref, Reflect, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[relationship(relationship_target = Actions<C>)]
pub struct ActionOf<C: Component> {
    #[deref]
    #[relationship]
    entity: Entity,
    #[reflect(ignore)]
    marker: PhantomData<C>,
}

impl<C: Component> ActionOf<C> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            marker: PhantomData,
        }
    }
}

/// Action entities associated with context `C`.
///
/// See also the [`actions!`] macro for conveniently spawning associated actions.
#[derive(Component, Deref, Reflect, Debug, Default, PartialEq, Eq)]
#[relationship_target(relationship = ActionOf<C>, linked_spawn)]
pub struct Actions<C: Component> {
    #[deref]
    #[relationship]
    entities: Vec<Entity>,
    #[reflect(ignore)]
    marker: PhantomData<C>,
}

impl<'a, C: Component> IntoIterator for &'a Actions<C> {
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

/// Returns a [`SpawnRelatedBundle`] that will insert the [`Actions<C>`] component and
/// spawn a [`SpawnableList`] of entities with given bundles that relate to the context entity via the [`ActionOf<C>`] component.
///
/// Similar to [`related!`], but instead of specifying [`Actions<C>`], you only write `C` itself.
///
/// See also [`bindings!`].
///
/// # Examples
///
/// Spawn a context with associated actions.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn((
///     OnFoot,
///     // Equivalent to `related!(Actions::<OnFoot>[`.
///     actions!(OnFoot[
///         (
///             Action::<Move>::new(),
///             Bindings::spawn(Cardinal::wasd_keys()),
///         ),
///         (
///             Action::<EnterCar>::new(),
///             ActionSettings { require_reset: true, Default::default() },
///             bindings![KeyCode::Enter],
///         ),
///     ])
/// ));
/// # #[derive(Component)]
/// # struct OnFoot;
/// # #[derive(InputAction)]
/// # #[action_output(Vec2)]
/// # struct Move;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct EnterCar;
/// ```
#[macro_export]
macro_rules! actions {
    ($context:ty [$($action:expr),*$(,)?]) => {
       $crate::prelude::Actions::<$context>::spawn(($(::bevy::prelude::Spawn($action)),*))
    };
}
