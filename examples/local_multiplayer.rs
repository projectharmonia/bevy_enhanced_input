//! Two players that use the same context type, but with different bindings.

use bevy::{input::gamepad::GamepadConnectionEvent, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .init_resource::<Gamepads>()
        .add_input_context::<Player>()
        .add_observer(bind)
        .add_observer(apply_movement)
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            update_gamepads.run_if(on_event::<GamepadConnectionEvent>),
        )
        .run();
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 25.0, 0.0).looking_at(-Vec3::Y, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Spawn two players with different assigned indices.
    let capsule = meshes.add(Capsule3d::new(0.5, 2.0));
    commands.spawn((
        Mesh3d(capsule.clone()),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 1.5, 8.0),
        Actions::<Player>::default(),
        Player::First,
    ));

    commands.spawn((
        Mesh3d(capsule),
        MeshMaterial3d(materials.add(Color::srgb_u8(220, 90, 90))),
        Transform::from_xyz(0.0, 1.5, -8.0),
        Actions::<Player>::default(),
        Player::Second,
    ));
}

fn bind(
    trigger: Trigger<Bind<Player>>,
    gamepads: Query<Entity, With<Gamepad>>,
    mut players: Query<(&Player, &mut Actions<Player>)>,
) {
    let (&player, mut actions) = players.get_mut(trigger.target()).unwrap();

    // By default actions read inputs from all gamepads,
    // but for local multiplayer we need assign specific
    // gamepad index. If no gamepad with the given exists,
    // use a placeholder to disable gamepad input.
    let gamepad_entity = gamepads.iter().nth(player as usize);
    actions.set_gamepad(gamepad_entity.unwrap_or(Entity::PLACEHOLDER));

    // Assign different bindings based player index.
    match player {
        Player::First => {
            actions
                .bind::<Move>()
                .to((Cardinal::wasd_keys(), Axial::left_stick()));
        }
        Player::Second => {
            actions
                .bind::<Move>()
                .to((Cardinal::arrow_keys(), Axial::left_stick()));
        }
    }

    // Can be called multiple times extend bindings.
    // In our case we add modifiers for all players.
    actions
        .bind::<Move>()
        .with_modifiers((DeadZone::default(), SmoothNudge::default()));
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();

    // Adjust axes for top-down movement.
    transform.translation.z -= trigger.value.x;
    transform.translation.x -= trigger.value.y;

    // Prevent from moving out of plane.
    transform.translation.z = transform.translation.z.clamp(-10.0, 10.0);
    transform.translation.x = transform.translation.x.clamp(-10.0, 10.0);
}

fn update_gamepads(mut commands: Commands) {
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
