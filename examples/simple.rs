//! Simple setup with a single context.

mod player_box;

use core::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBox, PlayerBoxPlugin, DEFAULT_SPEED};

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
        app.add_input_context::<PlayerBox>() // Attach input context to a component.
            .add_observer(binding) // Add observer to setup bindings.
            .add_observer(apply_movement)
            .add_observer(rotate)
            .add_systems(Startup, spawn);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn an entity with a component that implements `InputContext`.
    commands.spawn(PlayerBox);
}

// To define mappings for actions, write an observer for `Binding`.
// You can implement it for your character component directly, as
// shown in this example, if you don't plan to switch contexts.
fn binding(mut trigger: Trigger<Binding<PlayerBox>>) {
    // Mappings like WASD or sticks are very common,
    // so we provide built-ins to assign all keys/axes at once.
    // We don't assign any conditions and in this case the action will
    // be triggered with any non-zero value.
    trigger
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), GamepadStick::Left))
        .with_modifiers((
            DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            Scale::splat(DEFAULT_SPEED), // Additionally multiply by a constant to achieve the desired speed.
        ));

    // Multiple inputs can be assigned to a single action,
    // and the action will respond to any of them.
    trigger
        .bind::<Rotate>()
        .to((KeyCode::Space, GamepadButton::South));
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.entity()).unwrap();
    // The value has already been preprocessed by defined modifiers.
    transform.translation += trigger.value.extend(0.0);
}

fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.entity()).unwrap();
    transform.rotate_z(FRAC_PI_4);
}

// All actions should implement the `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
// The only necessary parameter is `output`, which defines the output type.
#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;
