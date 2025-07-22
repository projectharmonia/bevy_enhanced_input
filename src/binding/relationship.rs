use alloc::slice;
use core::iter::Copied;

use bevy::{
    ecs::relationship::{RelatedSpawner, RelatedSpawnerCommands},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Action entity associated with this binding entity.
///
/// See also the [`bindings!`](crate::prelude::bindings) macro for conveniently spawning associated actions.
#[derive(Component, Deref, Reflect, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[relationship(relationship_target = Bindings)]
pub struct BindingOf(pub Entity);

/// Binding entities associated with this action entity.
///
/// See also the [`bindings!`](crate::prelude::bindings) macro for conveniently spawning associated actions.
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

/// Returns a [`SpawnRelatedBundle`](bevy::ecs::spawn::SpawnRelatedBundle) that will insert the [`Bindings`] component and
/// spawn a [`SpawnableList`] of entities with given bundles that relate to the context entity via the
/// [`BindingOf`] component.
///
/// Similar to [`related!`], but allows you to omit the explicit [`Bindings`] type and automatically wraps elements using
/// [`Binding::from`](crate::prelude::Binding::from).
///
/// The macro accepts either individual elements that implement [`Into<Binding>`], or tuples where the first element implements
/// [`Into<Binding>`] and the remaining elements are bundles.
///
/// Due to `macro_rules!` limitations, you can't mix tuples and individual elements. However, you can wrap individual elements in braces
/// to use them alongside tuples.
///
/// The macro can't be used to spawn [presets](crate::preset). See the module documentation for more details.
///
/// See also [`actions!`](crate::prelude::actions).
///
/// # Examples
///
/// List of single elements.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # use core::any;
/// let from_macro = bindings![KeyCode::Space, GamepadButton::South];
/// // Expands to the following:
/// let manual = Bindings::spawn((
///     Spawn(Binding::from(KeyCode::Space)),
///     Spawn(Binding::from(GamepadButton::South)),
/// ));
///
/// assert_eq!(any::type_name_of_val(&from_macro), any::type_name_of_val(&manual));
/// ```
///
/// List of tuples.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # use core::any;
/// let from_macro = bindings![
///     (GamepadButton::RightTrigger2, Down::new(0.3)),
///     (MouseButton::Left), // Necessary to wrap in braces since we use a tuple above.
/// ];
/// // Expands to the following:
/// let manual = Bindings::spawn((
///     Spawn((Binding::from(GamepadButton::RightTrigger2), Down::new(0.3))),
///     Spawn((Binding::from(MouseButton::Left),)), // Extra braces could be omitted here, but necessary for the check below.
/// ));
///
/// assert_eq!(any::type_name_of_val(&from_macro), any::type_name_of_val(&manual));
/// ```
///
/// [`SpawnableList`]: bevy::ecs::spawn::SpawnableList
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
