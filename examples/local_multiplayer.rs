//! Two players that use the same context type, but with different bindings.

mod player_box;

use core::f32::consts::FRAC_PI_4;

use bevy::{
    color::palettes::tailwind::{BLUE_600, RED_600},
    input::gamepad::{GamepadConnection, GamepadConnectionEvent},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

use player_box::{DEFAULT_SPEED, PlayerBox, PlayerBoxPlugin, PlayerColor};

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
        app.add_input_context::<Player>()
            .add_observer(binding)
            .add_observer(apply_movement)
            .add_observer(rotate)
            .add_systems(Startup, spawn)
            .add_systems(Update, update_gamepads);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn two players with different assigned indices.
    commands.spawn((
        PlayerBox,
        Transform::from_translation(Vec3::X * 50.0),
        PlayerColor(RED_600.into()),
        Player::First,
        Actions::<Player>::default(),
    ));
    commands.spawn((
        PlayerBox,
        Transform::from_translation(-Vec3::X * 50.0),
        PlayerColor(BLUE_600.into()),
        Player::Second,
        Actions::<Player>::default(),
    ));
}

fn binding(
    trigger: Trigger<Binding<Player>>,
    gamepads: Res<Gamepads>,
    mut players: Query<(&Player, &mut Actions<Player>)>,
) {
    let (&player, mut actions) = players.get_mut(trigger.target()).unwrap();

    // By default actions read inputs from all gamepads,
    // but for local multiplayer we need assign specific
    // gamepad index.
    if let Some(&entity) = gamepads.get(player as usize) {
        actions.set_gamepad(entity);
    }

    // Assign different bindings based player index.
    match player {
        Player::First => {
            actions
                .bind::<Move>()
                .to((Cardinal::wasd_keys(), GamepadStick::Left));
            actions
                .bind::<Rotate>()
                .to((KeyCode::Space, GamepadButton::South));
        }
        Player::Second => {
            actions
                .bind::<Move>()
                .to((Cardinal::arrow_keys(), GamepadStick::Left));
            actions
                .bind::<Rotate>()
                .to((KeyCode::Numpad0, GamepadButton::South));
        }
    }

    // Can be called multiple times extend bindings.
    // In our case we add modifiers for all players.
    actions.bind::<Move>().with_modifiers((
        DeadZone::default(),
        SmoothNudge::default(),
        Scale::splat(DEFAULT_SPEED),
    ));
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();
    transform.translation += trigger.value.extend(0.0);
}

fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();
    transform.rotate_z(FRAC_PI_4);
}

fn update_gamepads(
    mut commands: Commands,
    mut connect_events: EventReader<GamepadConnectionEvent>,
    mut gamepads: ResMut<Gamepads>,
) {
    for event in connect_events.read() {
        match event.connection {
            GamepadConnection::Connected { .. } => gamepads.push(event.gamepad),
            GamepadConnection::Disconnected => {
                if let Some(index) = gamepads.iter().position(|&entity| entity == event.gamepad) {
                    gamepads.swap_remove(index);
                }
            }
        }
    }

    // Update associated gamepads.
    commands.trigger(RebuildBindings);
}

/// Used as both input context and component.
#[derive(InputContext, Component, Clone, Copy, PartialEq, Eq, Hash)]
enum Player {
    First,
    Second,
}

/// A resource that tracks all connected gamepads to pick them by index.
#[derive(Resource, Default, Deref, DerefMut)]
struct Gamepads(Vec<Entity>);

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;
