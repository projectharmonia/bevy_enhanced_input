use bevy::{ecs::spawn::SpawnableList, prelude::*};

use crate::prelude::*;

/// A preset to map 2 buttons as 1-dimensional input.
///
/// See [`Cardinal`] for a usage example.
#[derive(Debug, Clone, Copy)]
pub struct Bidirectional<P: Bundle, N: Bundle> {
    pub positive: P,
    pub negative: N,
}

impl<P: Bundle, N: Bundle> Bidirectional<P, N> {
    pub fn with<T: Bundle + Clone>(self, bundle: T) -> Bidirectional<(P, T), (N, T)> {
        Bidirectional {
            positive: (self.positive, bundle.clone()),
            negative: (self.negative, bundle),
        }
    }
}

impl<P: Bundle, N: Bundle> SpawnableList<BindingOf> for Bidirectional<P, N> {
    fn spawn(self, world: &mut World, entity: Entity) {
        world.spawn((BindingOf(entity), self.positive));
        world.spawn((BindingOf(entity), self.negative, Negate::all()));
    }

    fn size_hint(&self) -> usize {
        2
    }
}
