//! One context completely replaces another.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_input_context::<Inventory>()
        .add_observer(apply_movement)
        .add_observer(attack)
        .add_observer(open_inventory)
        .add_observer(navigate_inventory)
        .add_observer(close_inventory)
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn(player());
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
        .remove::<Player>()
        .despawn_related::<Actions<Player>>()
        .insert((
            Inventory,
            actions!(Inventory[
                (
                    Action::<NavigateInventory>::new(),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                    Pulse::new(0.2), // Avoid triggering every frame on hold for UI.
                ),
                (
                    Action::<CloseInventory>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
                    bindings![KeyCode::KeyI, GamepadButton::Select],
                )
            ]),
        ));
}

fn navigate_inventory(_trigger: Trigger<Fired<NavigateInventory>>) {
    info!("navigating inventory");
}

fn close_inventory(trigger: Trigger<Started<CloseInventory>>, mut commands: Commands) {
    info!("closing inventory");
    commands
        .entity(trigger.target())
        .remove::<Inventory>()
        .despawn_related::<Actions<Inventory>>()
        .insert(player());
}

fn player() -> impl Bundle {
    (
        Player,
        actions!(Player[
            (
                Action::<Move>::new(),
                DeadZone::default(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (
                Action::<Attack>::new(),
                bindings![MouseButton::Left, GamepadButton::West],
            ),
            (
                Action::<OpenInventory>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::KeyI, GamepadButton::Select],
            ),
        ]),
    )
}

#[derive(Component, InputContext)]
struct Player;

#[derive(InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(InputAction)]
#[input_action(output = bool)]
struct Attack;

/// Switches context to [`Inventory`].
///
/// We set `require_reset` to `true` because [`CloseInventory`] action uses the same input,
/// and we want it to be triggerable only after the button is released.
#[derive(InputAction)]
#[input_action(output = bool)]
struct OpenInventory;

#[derive(Component, InputContext)]
struct Inventory;

#[derive(InputAction)]
#[input_action(output = Vec2)]
struct NavigateInventory;

/// Switches context to [`Player`].
///
/// See [`OpenInventory`] for details about `require_reset`.
#[derive(InputAction)]
#[input_action(output = bool)]
struct CloseInventory;
