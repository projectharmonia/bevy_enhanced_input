//! One context completely replaces another.

mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::{color::palettes::tailwind::FUCHSIA_400, prelude::*};
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBoxBundle, PlayerBoxPlugin, PlayerColor, DEFAULT_SPEED};

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
        app.add_input_context::<OnFoot>()
            .add_input_context::<InCar>()
            .add_systems(Startup, Self::spawn)
            .observe(Self::apply_movement)
            .observe(Self::rotate)
            .observe(Self::enter_car)
            .observe(Self::exit_car);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
        commands.spawn((PlayerBoxBundle::default(), OnFoot));
    }

    fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.translation += event.value.as_axis3d();
    }

    fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.rotate_z(FRAC_PI_4);
    }

    fn enter_car(
        trigger: Trigger<Started<EnterCar>>,
        mut commands: Commands,
        mut players: Query<&mut PlayerColor>,
    ) {
        // Change color for visibility.
        let mut color = players.get_mut(trigger.entity()).unwrap();
        **color = FUCHSIA_400.into();

        commands
            .entity(trigger.entity())
            .remove::<OnFoot>()
            .insert(InCar);
    }

    fn exit_car(
        trigger: Trigger<Started<ExitCar>>,
        mut commands: Commands,
        mut players: Query<&mut PlayerColor>,
    ) {
        let mut color = players.get_mut(trigger.entity()).unwrap();
        **color = Default::default();

        commands
            .entity(trigger.entity())
            .remove::<InCar>()
            .insert(OnFoot);
    }
}

#[derive(Component)]
struct OnFoot;

impl InputContext for OnFoot {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut instance = ContextInstance::default();

        instance
            .bind::<Move>()
            .with(WasdKeys)
            .with_modifier(DeadZone::default())
            .with_modifier(DeltaLerp::default())
            .with_modifier(Scale::splat(DEFAULT_SPEED));
        instance.bind::<Rotate>().with(KeyCode::Space);
        instance.bind::<EnterCar>().with(KeyCode::Enter);

        instance
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Rotate;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct EnterCar;

#[derive(Component)]
struct InCar;

impl InputContext for InCar {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>()
            .with(WasdKeys)
            .with_modifier(DeadZone::default())
            .with_modifier(DeltaLerp::default())
            .with_modifier(Scale::splat(DEFAULT_SPEED + 20.0)); // Make car faster.
        ctx.bind::<ExitCar>().with(KeyCode::Enter);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct ExitCar;
