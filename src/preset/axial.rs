use bevy::{ecs::spawn::SpawnableList, prelude::*};

use crate::prelude::*;

/// A preset to map 2 axes as 2-dimensional input.
///
/// # Examples
///
/// Maps gamepad axes into a 2D movement action.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn bind(
///     trigger: Trigger<Bind<Player>>,
///     settings: Res<GamepadSettings>,
///     mut players: Query<&mut Actions<Player>>,
/// ) {
///     let mut actions = players.get_mut(trigger.target()).unwrap();
///     actions.bind::<Move>().to(Axial {
///         x: &settings.horizontal_movement,
///         y: &settings.vertical_movement,
///     });
/// }
///
/// #[derive(Resource)]
/// struct GamepadSettings {
///     horizontal_movement: Vec<GamepadAxis>,
///     vertical_movement: Vec<GamepadAxis>,
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
pub struct Axial<X, Y> {
    pub x: X,
    pub y: Y,
}

impl<X, Y, T: Clone> WithBundle<T> for Axial<X, Y> {
    type Output = Axial<(X, T), (Y, T)>;

    fn with(self, bundle: T) -> Self::Output {
        Axial {
            x: (self.x, bundle.clone()),
            y: (self.y, bundle),
        }
    }
}

impl Axial<Binding, Binding> {
    /// Maps left stick as 2-dimensional input.
    pub fn left_stick() -> Self {
        Self {
            x: GamepadAxis::LeftStickX.into(),
            y: GamepadAxis::LeftStickY.into(),
        }
    }

    /// Maps right stick as 2-dimensional input.
    pub fn right_stick() -> Self {
        Self {
            x: GamepadAxis::RightStickX.into(),
            y: GamepadAxis::RightStickY.into(),
        }
    }
}

impl<X: Bundle, Y: Bundle> SpawnableList<BindingOf> for Axial<X, Y> {
    fn spawn(self, world: &mut World, entity: Entity) {
        world.spawn((BindingOf(entity), self.x));
        world.spawn((BindingOf(entity), self.y, SwizzleAxis::YXZ));
    }

    fn size_hint(&self) -> usize {
        2
    }
}
