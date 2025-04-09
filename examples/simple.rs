//! Simple setup with a single context.

mod player_box;

use core::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use player_box::{DEFAULT_SPEED, PlayerBox, PlayerBoxPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EnhancedInputPlugin,
            PlayerBoxPlugin,
            GamePlugin,
        ))
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>() // All contexts should be registered.
            .add_observer(binding) // Add observer to setup bindings.
            .add_observer(apply_movement)
            .add_observer(rotate)
            .add_systems(Startup, spawn);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn an entity with `Actions` component.
    commands.spawn((PlayerBox, Actions::<Player>::default()));
}

// To define bindings for actions, write an observer for `Binding`.
// It's also possible to create bindings before the insertion,
// but this way you can conveniently reload bindings when settings change.
fn binding(trigger: Trigger<Binding<Player>>, mut players: Query<&mut Actions<Player>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();

    // Bindings like WASD or sticks are very common,
    // so we provide built-ins to assign all keys/axes at once.
    // We don't assign any conditions and in this case the action will
    // be triggered with any non-zero value.
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), GamepadStick::Left))
        .with_modifiers((
            DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            Scale::splat(DEFAULT_SPEED), // Additionally multiply by a constant to achieve the desired speed.
        ));

    // Multiple inputs can be assigned to a single action,
    // and the action will respond to any of them.
    actions
        .bind::<Rotate>()
        .to((KeyCode::Space, GamepadButton::South));
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();
    // The value has already been preprocessed by defined modifiers.
    transform.translation += trigger.value.extend(0.0);
}

fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();
    transform.rotate_z(FRAC_PI_4);
}

// Since it's possible to have multiple `Actions` components, you need
// to define a marker and derive `InputContext` trait.
#[derive(InputContext)]
struct Player;

// All actions should implement the `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
// The only necessary parameter is `output`, which defines the output type.
#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;
