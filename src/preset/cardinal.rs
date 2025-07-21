use bevy::{ecs::spawn::SpawnableList, prelude::*};

use crate::prelude::*;

/**
A preset to 4 map buttons as 2-dimensional input.

In Bevy's 3D space, the -Z axis points forward and the +Z axis points
toward the camera. To map movement correctly in 3D space for [`Transform::translation`],
you will need to invert Y and apply it to Z inside your observer.

# Examples

Attach modifiers to fields.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

# let mut world = World::new();
world.spawn((
    Player,
    actions!(Player[
        (
            Action::<Move>::new(),
            Bindings::spawn((
                Cardinal::wasd_keys().with(Scale::splat(2.0)), // You can attach modifiers to all fields thanks to `WithBundle` impl.
                Cardinal {
                    // The struct consists of bundles, so you can also attach modifiers to individual fields.
                    north: Binding::from(KeyCode::KeyI),
                    east: (Binding::from(KeyCode::KeyL), Scale::splat(2.0)),
                    south: Binding::from(KeyCode::KeyK),
                    west: (Binding::from(KeyCode::KeyJ), Scale::splat(2.0)),
                },
            )),
        ),
    ]),
));

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;
```

Loading from settings.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::{Serialize, Deserialize};

// Could be loaded from a file.
// `Binding::None` represents unbound inputs.
let settings = InputSettings {
    forward: [Binding::from(KeyCode::KeyW), Binding::None],
    right: [Binding::from(KeyCode::KeyA), Binding::None],
    backward: [Binding::from(KeyCode::KeyS), Binding::None],
    left: [Binding::from(KeyCode::KeyD), Binding::None],
};

# let mut world = World::new();
world.spawn((
    Player,
    actions!(Player[
        (
            Action::<Move>::new(),
            Bindings::spawn((
                Cardinal {
                    north: settings.forward[0],
                    east: settings.right[0],
                    south: settings.backward[0],
                    west: settings.left[0],
                },
                Cardinal {
                    north: settings.forward[1],
                    east: settings.right[1],
                    south: settings.backward[1],
                    west: settings.left[1],
                },
            )),
        ),
    ]),
));

/// Bindings for actions.
///
/// Represented as arrays because in games you usually
/// have 2 or 3 bindings for a single action.
///
/// Usually stored as a resource.
#[derive(Resource, Serialize, Deserialize)]
struct InputSettings {
    forward: [Binding; 2],
    right: [Binding; 2],
    backward: [Binding; 2],
    left: [Binding; 2],
}

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;
```
*/
#[derive(Debug, Clone, Copy)]
pub struct Cardinal<N, E, S, W> {
    pub north: N,
    pub east: E,
    pub south: S,
    pub west: W,
}

impl<N, E, S, W, T: Clone> WithBundle<T> for Cardinal<N, E, S, W> {
    type Output = Cardinal<(N, T), (E, T), (S, T), (W, T)>;

    fn with(self, bundle: T) -> Self::Output {
        Cardinal {
            north: (self.north, bundle.clone()),
            east: (self.east, bundle.clone()),
            south: (self.south, bundle.clone()),
            west: (self.west, bundle),
        }
    }
}

impl Cardinal<Binding, Binding, Binding, Binding> {
    /// Maps WASD keys as 2-dimensional input.
    #[must_use]
    pub fn wasd_keys() -> Self {
        Self {
            north: KeyCode::KeyW.into(),
            west: KeyCode::KeyA.into(),
            south: KeyCode::KeyS.into(),
            east: KeyCode::KeyD.into(),
        }
    }

    /// Maps keyboard arrow keys as 2-dimensional input.
    #[must_use]
    pub fn arrow_keys() -> Self {
        Self {
            north: KeyCode::ArrowUp.into(),
            west: KeyCode::ArrowLeft.into(),
            south: KeyCode::ArrowDown.into(),
            east: KeyCode::ArrowRight.into(),
        }
    }
}

impl Cardinal<Binding, Binding, Binding, Binding> {
    /// Maps D-pad as 2-dimensional input.
    #[must_use]
    pub fn dpad_buttons() -> Self {
        Self {
            north: GamepadButton::DPadUp.into(),
            west: GamepadButton::DPadLeft.into(),
            south: GamepadButton::DPadDown.into(),
            east: GamepadButton::DPadRight.into(),
        }
    }
}

impl<N: Bundle, E: Bundle, S: Bundle, W: Bundle> SpawnableList<BindingOf> for Cardinal<N, E, S, W> {
    fn spawn(self, world: &mut World, entity: Entity) {
        let x = Bidirectional {
            positive: self.east,
            negative: self.west,
        };
        x.spawn(world, entity);

        let y = Bidirectional {
            positive: self.north,
            negative: self.south,
        };
        y.with(SwizzleAxis::YXZ).spawn(world, entity);
    }

    fn size_hint(&self) -> usize {
        4
    }
}
