use bevy::{ecs::spawn::SpawnableList, prelude::*};

use crate::prelude::*;

/// A preset to 4 map buttons as 2-dimensional input.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space for [`Transform::translation`],
/// you will need to invert Y and apply it to Z inside your observer.
///
/// # Examples
///
/// Map keyboard inputs into a 2D movement action.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn bind(
///     trigger: Trigger<Bind<Player>>,
///     settings: Res<KeyboardSettings>,
///     mut players: Query<&mut Actions<Player>>,
/// ) {
///     let mut actions = players.get_mut(trigger.target()).unwrap();
///     actions.bind::<Move>().to(Cardinal {
///         north: &settings.forward,
///         east: &settings.right,
///         south: &settings.backward,
///         west: &settings.left,
///     });
/// }
///
/// // We use `KeyCode` here because we are only interested in key presses.
/// // But you can also use `Input` if you want to e.g.
/// // combine mouse and keyboard input sources.
/// #[derive(Resource)]
/// struct KeyboardSettings {
///     forward: Vec<KeyCode>,
///     right: Vec<KeyCode>,
///     backward: Vec<KeyCode>,
///     left: Vec<KeyCode>,
/// }
///
/// #[derive(InputContext)]
/// struct Player;
///
/// #[derive(InputAction)]
/// #[action_output(Vec2)]
/// struct Move;
/// ```
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
