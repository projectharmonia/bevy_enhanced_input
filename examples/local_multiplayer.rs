//! Two players that use the same context type, but with different mappings.

mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::{
    color::palettes::tailwind::{BLUE_600, RED_600},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBox, PlayerBoxBundle, PlayerBoxPlugin, PlayerColor, DEFAULT_SPEED};

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
        app.add_input_context::<PlayerBox>()
            .add_systems(Startup, Self::spawn)
            .observe(Self::apply_movement)
            .observe(Self::rotate);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());

        // Spawn two players with different assigned indices.
        commands.spawn((
            PlayerBoxBundle {
                transform: Transform::from_translation(Vec3::X * 50.0),
                color: PlayerColor(RED_600.into()),
                ..Default::default()
            },
            PlayerIndex(0),
        ));
        commands.spawn((
            PlayerBoxBundle {
                transform: Transform::from_translation(-Vec3::X * 50.0),
                color: PlayerColor(BLUE_600.into()),
                ..Default::default()
            },
            PlayerIndex(1),
        ));
    }

    fn apply_movement(trigger: Trigger<ActionEvent<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        if event.is_fired() {
            let mut transform = players.get_mut(trigger.entity()).unwrap();
            transform.translation += event.value.as_axis3d();
        }
    }

    fn rotate(trigger: Trigger<ActionEvent<Rotate>>, mut players: Query<&mut Transform>) {
        if trigger.event().is_started() {
            let mut transform = players.get_mut(trigger.entity()).unwrap();
            transform.rotate_z(FRAC_PI_4);
        }
    }
}

#[derive(Component, Deref)]
struct PlayerIndex(usize);

impl InputContext for PlayerBox {
    fn context_instance(world: &World, entity: Entity) -> ContextInstance {
        // Could be stored in the context itself, but it's usually
        // better to have a separate component that is shared
        // across all contexts.
        let index = **world.get::<PlayerIndex>(entity).unwrap();

        // By default context read inputs from all gamepads,
        // but for local multiplayer we need assign specific
        // gamepad index.
        let mut ctx = ContextInstance::with_gamepad(index);

        // Assign different mappings based player index.
        match index {
            0 => {
                ctx.bind::<Move>()
                    .with_wasd()
                    .with_stick(GamepadStick::Left);
                ctx.bind::<Rotate>()
                    .with(KeyCode::Space)
                    .with(GamepadButtonType::South);
            }
            1 => {
                ctx.bind::<Move>()
                    .with_arrows()
                    .with_stick(GamepadStick::Left);
                ctx.bind::<Rotate>()
                    .with(KeyCode::Numpad0)
                    .with(GamepadButtonType::South);
            }
            _ => {
                panic!("game expects only 2 players");
            }
        }

        // Can be called multiple times extend bindings.
        // In our case we cant to add modifiers for all players.
        ctx.bind::<Move>()
            .with_modifier(Normalize)
            .with_modifier(DeltaLerp::default())
            .with_modifier(Scale::splat(DEFAULT_SPEED));

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Rotate;
