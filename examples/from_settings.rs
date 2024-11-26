//! Creates a context from a settings struct.

mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_enhanced_input::{input_context::bind::BindConfigs, prelude::*};

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

        // Spawn an entity with a component that implements `InputContext`.
        commands.spawn(PlayerBoxBundle::default());
    }

    fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        // The value has already been preprocessed by defined modifiers.
        transform.translation += event.value.as_axis3d();
    }

    fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.rotate_z(FRAC_PI_4);
    }
}

#[derive(Debug, Resource)]
struct ControlSettings {
    up: Vec<Input>,
    right: Vec<Input>,
    down: Vec<Input>,
    left: Vec<Input>,
    rotate: Vec<Input>,
}

impl Default for ControlSettings {
    fn default() -> Self {
        Self {
            up: vec![KeyCode::KeyW.into()],
            right: vec![KeyCode::KeyD.into()],
            down: vec![KeyCode::KeyS.into()],
            left: vec![KeyCode::KeyA.into()],
            rotate: vec![KeyCode::Space.into()],
        }
    }
}

impl InputContext for PlayerBox {
    fn context_instance(world: &World, _entity: Entity) -> ContextInstance {
        let settings = world.resource::<ControlSettings>();

        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>()
            .to(Cardinal {
                north: settings.up.clone(),
                east: settings.right.clone(),
                south: settings.down.clone(),
                west: settings.left.clone(),
            })
            .with_modifier(DeadZone::default())
            .with_modifier(DeltaLerp::default())
            .with_modifier(Scale::splat(DEFAULT_SPEED));

        ctx.bind::<Rotate>().to(settings.rotate.clone());

        ctx
    }
}

// All actions should implement the `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
// The only necessary parameter is `dim`, which defines the output type.
#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Rotate;
