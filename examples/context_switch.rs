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
    commands.spawn((PlayerBox, Actions::<OnFoot>::default()));
}

fn foot_binding(trigger: Trigger<Binding<OnFoot>>, mut players: Query<&mut Actions<OnFoot>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
            Scale::splat(DEFAULT_SPEED),
        ));
    actions.bind::<Rotate>().to(KeyCode::Space);
    actions.bind::<EnterCar>().to(KeyCode::Enter);
}

fn car_binding(trigger: Trigger<Binding<InCar>>, mut players: Query<&mut Actions<InCar>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
            Scale::splat(DEFAULT_SPEED + 20.0), // Make car faster.
        ));
    actions.bind::<ExitCar>().to(KeyCode::Enter);
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();
    transform.translation += trigger.value.extend(0.0);
}

fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();
    transform.rotate_z(FRAC_PI_4);
}

fn enter_car(
    trigger: Trigger<Started<EnterCar>>,
    mut commands: Commands,
    mut players: Query<&mut PlayerColor>,
) {
    // Change color for visibility.
    let mut color = players.get_mut(trigger.target()).unwrap();
    **color = FUCHSIA_400.into();

    commands
        .entity(trigger.target())
        .remove::<Actions<OnFoot>>()
        .insert(Actions::<InCar>::default());
}

fn exit_car(
    trigger: Trigger<Started<ExitCar>>,
    mut commands: Commands,
    mut players: Query<&mut PlayerColor>,
) {
    let mut color = players.get_mut(trigger.target()).unwrap();
    **color = Default::default();

    commands
        .entity(trigger.target())
        .remove::<Actions<InCar>>()
        .insert(Actions::<OnFoot>::default());
}

#[derive(InputContext)]
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

#[derive(InputContext)]
struct InCar;

/// Switches context to [`OnFoot`].
///
/// See [`EnterCar`] for details about `require_reset`.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct ExitCar;
