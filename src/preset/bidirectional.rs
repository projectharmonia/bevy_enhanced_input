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
    pub fn with<A: Bundle + Clone>(self, bundle: A) -> Bidirectional<(P, A), (N, A)> {
        Bidirectional::<(P, A), (N, A)> {
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
