//! One context completely replaces another.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_input_context::<Inventory>()
        .add_observer(player_binding)
        .add_observer(inventory_binding)
        .add_observer(apply_movement)
        .add_observer(attack)
        .add_observer(open_inventory)
        .add_observer(navigate_inventory)
        .add_observer(close_inventory)
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn(Actions::<Player>::default());
}

fn player_binding(trigger: Trigger<Binding<Player>>, mut players: Query<&mut Actions<Player>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers(DeadZone::default());
    actions
        .bind::<Attack>()
        .to((MouseButton::Left, GamepadButton::West));
    actions
        .bind::<OpenInventory>()
        .to((KeyCode::KeyI, GamepadButton::Select));
}

fn inventory_binding(
    trigger: Trigger<Binding<Inventory>>,
    mut players: Query<&mut Actions<Inventory>>,
) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<NavigateInventory>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_conditions(Pulse::new(0.2)); // Avoid triggering every frame on hold for UI.
    actions
        .bind::<CloseInventory>()
        .to((KeyCode::KeyI, GamepadButton::Select));
}

fn apply_movement(trigger: Trigger<Fired<Move>>) {
    info!("moving: {}", trigger.value);
}

fn attack(_trigger: Trigger<Fired<Attack>>) {
    info!("attacking");
}

fn open_inventory(trigger: Trigger<Started<OpenInventory>>, mut commands: Commands) {
    info!("opening inventory");
    commands
        .entity(trigger.target())
        .remove::<Actions<Player>>()
        .insert(Actions::<Inventory>::default());
}

fn navigate_inventory(_trigger: Trigger<Fired<NavigateInventory>>) {
    info!("navigating inventory");
}

fn close_inventory(trigger: Trigger<Started<CloseInventory>>, mut commands: Commands) {
    info!("closing inventory");
    commands
        .entity(trigger.target())
        .remove::<Actions<Inventory>>()
        .insert(Actions::<Player>::default());
}

#[derive(InputContext)]
struct Player;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Attack;

/// Switches context to [`Inventory`].
///
/// We set `require_reset` to `true` because [`CloseInventory`] action uses the same input,
/// and we want it to be triggerable only after the button is released.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct OpenInventory;

#[derive(InputContext)]
struct Inventory;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct NavigateInventory;

/// Switches context to [`Player`].
///
/// See [`OpenInventory`] for details about `require_reset`.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct CloseInventory;
