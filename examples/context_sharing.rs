//! Two entities that share a single context.
//! This could be used for games where you control multiple characters at the same time,
//! such as Binary Land for NES.

mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBox, PlayerBoxBundle, PlayerBoxPlugin, DEFAULT_SPEED};

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

        // Spawn two entities with the same context.
        commands.spawn(PlayerBoxBundle {
            transform: Transform::from_translation(Vec3::X * 50.0),
            ..Default::default()
        });
        commands.spawn(PlayerBoxBundle {
            transform: Transform::from_translation(-Vec3::X * 50.0),
            ..Default::default()
        });
    }

    fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.translation += event.value.extend(0.0);
    }

    fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.rotate_z(FRAC_PI_4);
    }
}

impl InputContext for PlayerBox {
    // By default all context instances are processed individually.
    // This means if multiple entities spawned with the same mappings,
    // actions from the first processed context may consume inputs.
    // Make it shared to have a single context instance for all entities
    // with this context.
    const MODE: ContextMode = ContextMode::Shared;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>()
            .to(Cardinal::wasd_keys())
            .with_modifier(DeadZone::default())
            .with_modifier(DeltaLerp::default())
            .with_modifier(Scale::splat(DEFAULT_SPEED));
        ctx.bind::<Rotate>().to(KeyCode::Space);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;
