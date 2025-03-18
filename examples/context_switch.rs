//! One context completely replaces another.

mod player_box;

use core::f32::consts::FRAC_PI_4;

use bevy::{color::palettes::tailwind::FUCHSIA_400, prelude::*};
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
        app.add_input_context::<OnFoot>()
            .add_input_context::<InCar>()
            .add_observer(foot_binding)
            .add_observer(car_binding)
            .add_observer(apply_movement)
            .add_observer(rotate)
            .add_observer(enter_car)
            .add_observer(exit_car)
            .add_systems(Startup, spawn);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((PlayerBox, OnFoot));
}

fn foot_binding(mut trigger: Trigger<Binding<OnFoot>>) {
    trigger
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
            Scale::splat(DEFAULT_SPEED),
        ));
    trigger.bind::<Rotate>().to(KeyCode::Space);
    trigger.bind::<EnterCar>().to(KeyCode::Enter);
}

fn car_binding(mut trigger: Trigger<Binding<InCar>>) {
    trigger
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
            Scale::splat(DEFAULT_SPEED + 20.0), // Make car faster.
        ));
    trigger.bind::<ExitCar>().to(KeyCode::Enter);
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.entity()).unwrap();
    transform.translation += trigger.value.extend(0.0);
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

#[derive(Component)]
struct OnFoot;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;

/// Switches context to [`InCar`].
///
/// We set `require_reset` to `true` because [`ExitCar`] action uses the same input,
/// and we want it to be triggerable only after the button is released.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct EnterCar;

#[derive(Component)]
struct InCar;

/// Switches context to [`OnFoot`].
///
/// See [`EnterCar`] for details about `require_reset`.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct ExitCar;
