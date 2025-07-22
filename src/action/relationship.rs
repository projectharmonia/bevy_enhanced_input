use alloc::slice;
use core::{iter::Copied, marker::PhantomData};

use bevy::{
    ecs::relationship::{RelatedSpawner, RelatedSpawnerCommands},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Context entity associated with this action entity.
///
/// See also the [`actions!`](crate::prelude::actions) macro for conveniently spawning associated actions.
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
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            marker: PhantomData,
        }
    }
}

/// Action entities associated with context `C`.
///
/// See also the [`actions!`](crate::prelude::actions) macro for conveniently spawning associated actions.
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

/// Returns a [`SpawnRelatedBundle`](bevy::ecs::spawn::SpawnRelatedBundle) that will insert the [`Actions<C>`] component and
/// spawn a [`SpawnableList`](bevy::ecs::spawn::SpawnableList) of entities with given bundles that relate to the context entity
/// via the [`ActionOf<C>`] component.
///
/// Similar to [`related!`], but instead of specifying [`Actions<C>`], you only write `C` itself.
///
/// See also [`bindings!`](crate::prelude::bindings).
///
/// # Examples
///
/// List of single elements. You usually spawn actions with at least [`Bindings`](crate::prelude::Bindings),
/// but actions alone could be used for networking or for later mocking.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # use core::any;
/// let from_macro = actions!(Player[
///     Action::<Fire>::new(),
///     Action::<Jump>::new()
/// ]);
/// // Expands to the following:
/// let manual = Actions::<Player>::spawn((
///     Spawn(Action::<Fire>::new()),
///     Spawn(Action::<Jump>::new()),
/// ));
///
/// assert_eq!(any::type_name_of_val(&from_macro), any::type_name_of_val(&manual));
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Fire;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Jump;
/// ```
///
/// With tuples.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # use core::any;
/// let from_macro = actions!(Player[
///     (
///         Action::<Move>::new(),
///         Bindings::spawn(Cardinal::wasd_keys())
///     ),
///     Action::<Jump>::new(), // Unlike with `bindings!`, single values could be mixed with tuples.
/// ]);
/// // Expands to the following:
/// let manual = Actions::<Player>::spawn((
///     Spawn((Action::<Move>::new(), Bindings::spawn(Cardinal::wasd_keys()))),
///     Spawn(Action::<Jump>::new()),
/// ));
///
/// assert_eq!(any::type_name_of_val(&from_macro), any::type_name_of_val(&manual));
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(Vec2)]
/// # struct Move;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Jump;
/// ```
#[macro_export]
macro_rules! actions {
    ($context:ty [$($action:expr),*$(,)?]) => {
       $crate::prelude::Actions::<$context>::spawn(($(::bevy::prelude::Spawn($action)),*))
    };
}
