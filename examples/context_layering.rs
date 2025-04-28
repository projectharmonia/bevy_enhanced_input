//! One context applied on top of another and overrides some of the bindings.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_input_context::<Driving>()
        .add_observer(regular_binding)
        .add_observer(swimming_binding)
        .add_observer(apply_movement)
        .add_observer(jump)
        .add_observer(exit_car)
        .add_observer(enter_car)
        .add_observer(brake)
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn(Actions::<Player>::default());
}

fn regular_binding(trigger: Trigger<Binding<Player>>, mut players: Query<&mut Actions<Player>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers(DeadZone::default());
    actions
        .bind::<Jump>()
        .to((KeyCode::Space, GamepadButton::South));
    actions
        .bind::<EnterCar>()
        .to((KeyCode::Enter, GamepadButton::North));
}

fn swimming_binding(trigger: Trigger<Binding<Driving>>, mut players: Query<&mut Actions<Driving>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    // `Player` has lower priority, so `Brake` and `ExitCar` consume inputs first,
    // preventing `Rotate` and `EnterWater` from being triggered.
    // The consuming behavior can be configured in the `InputAction` trait.
    actions
        .bind::<Brake>()
        .to((KeyCode::Space, GamepadButton::East));
    actions
        .bind::<ExitCar>()
        .to((KeyCode::Enter, GamepadButton::North));
}

fn apply_movement(trigger: Trigger<Fired<Move>>) {
    info!("moving: {}", trigger.value);
}

fn jump(_trigger: Trigger<Started<Jump>>) {
    info!("jumping");
}

fn enter_car(trigger: Trigger<Started<EnterCar>>, mut commands: Commands) {
    info!("entering car");
    commands
        .entity(trigger.target())
        .insert(Actions::<Driving>::default());
}

fn brake(_trigger: Trigger<Fired<Brake>>) {
    info!("braking");
}

fn exit_car(trigger: Trigger<Started<ExitCar>>, mut commands: Commands) {
    info!("exiting car");
    commands
        .entity(trigger.target())
        .remove::<Actions<Driving>>();
}

#[derive(InputContext)]
struct Player;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Jump;

/// Adds [`Driving`].
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct EnterCar;

/// Overrides some actions from [`Player`].
#[derive(InputContext)]
#[input_context(priority = 1)]
struct Driving;

/// This action overrides [`Jump`] when the player is [`Driving`].
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Brake;

/// Removes [`Driving`].
///
/// We set `require_reset` to `true` because [`EnterWater`] action uses the same input,
/// and we want it to be triggerable only after the button is released.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct ExitCar;
