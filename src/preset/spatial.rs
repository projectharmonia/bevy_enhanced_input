use bevy::{ecs::spawn::SpawnableList, prelude::*};

use crate::prelude::*;

/// A preset to map 6 buttons as 3-dimensional input.
///
/// See [`Cardinal`] for a usage example.
#[derive(Debug, Clone, Copy)]
pub struct Spatial<F, B, L, R, U, D>
where
    F: Bundle,
    B: Bundle,
    L: Bundle,
    R: Bundle,
    U: Bundle,
    D: Bundle,
{
    pub forward: F,
    pub backward: B,
    pub left: L,
    pub right: R,
    pub up: U,
    pub down: D,
}

impl<F, B, L, R, U, D> Spatial<F, B, L, R, U, D>
where
    F: Bundle,
    B: Bundle,
    L: Bundle,
    R: Bundle,
    U: Bundle,
    D: Bundle,
{
    pub fn with<T: Bundle + Clone>(
        self,
        bundle: T,
    ) -> Spatial<(F, T), (B, T), (L, T), (R, T), (U, T), (D, T)> {
        Spatial::<(F, T), (B, T), (L, T), (R, T), (U, T), (D, T)> {
            forward: (self.forward, bundle.clone()),
            backward: (self.backward, bundle.clone()),
            left: (self.left, bundle.clone()),
            right: (self.right, bundle.clone()),
            up: (self.up, bundle.clone()),
            down: (self.down, bundle),
        }
    }
}

impl Spatial<Binding, Binding, Binding, Binding, Binding, Binding> {
    /// Maps WASD keys for horizontal (XZ) inputs and takes in up/down mappings.
    pub fn wasd_and(up: KeyCode, down: KeyCode) -> Self {
        Spatial {
            forward: KeyCode::KeyW.into(),
            backward: KeyCode::KeyS.into(),
            left: KeyCode::KeyA.into(),
            right: KeyCode::KeyD.into(),
            up: up.into(),
            down: down.into(),
        }
    }

    /// Maps arrow keys for horizontal (XZ) inputs and takes in up/down mappings.
    pub fn arrows_and(up: KeyCode, down: KeyCode) -> Self {
        Spatial {
            forward: KeyCode::ArrowUp.into(),
            backward: KeyCode::ArrowDown.into(),
            left: KeyCode::ArrowLeft.into(),
            right: KeyCode::ArrowRight.into(),
            up: up.into(),
            down: down.into(),
        }
    }
}

impl<F, B, L, R, U, D> SpawnableList<BindingOf> for Spatial<F, B, L, R, U, D>
where
    F: Bundle,
    B: Bundle,
    L: Bundle,
    R: Bundle,
    U: Bundle,
    D: Bundle,
{
    fn spawn(self, world: &mut World, entity: Entity) {
        let xy = Cardinal {
            north: self.up,
            east: self.right,
            south: self.down,
            west: self.left,
        };
        xy.spawn(world, entity);

        let z = Bidirectional {
            positive: self.backward,
            negative: self.forward,
        };
        z.with(SwizzleAxis::ZYX).spawn(world, entity);
    }

    fn size_hint(&self) -> usize {
        6
    }
}
