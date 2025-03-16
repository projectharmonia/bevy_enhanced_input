//! One context applied on top of another and overrides some of the mappings.

mod player_box;

use core::f32::consts::FRAC_PI_4;

use bevy::{color::palettes::tailwind::INDIGO_600, prelude::*};
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBox, PlayerBoxPlugin, PlayerColor, DEFAULT_SPEED};

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
            .add_input_context::<Swimming>()
            .add_observer(regular_binding)
            .add_observer(swimming_binding)
            .add_observer(apply_movement)
            .add_observer(rotate)
            .add_observer(exit_water)
            .add_observer(enter_water)
            .add_observer(start_diving)
            .add_observer(end_diving)
            .add_systems(Startup, spawn);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(PlayerBox);
}

fn regular_binding(mut trigger: Trigger<Binding<PlayerBox>>) {
    trigger
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
            Scale::splat(DEFAULT_SPEED),
        ));
    trigger.bind::<Rotate>().to(KeyCode::Space);
    trigger.bind::<EnterWater>().to(KeyCode::Enter);
}

fn swimming_binding(mut trigger: Trigger<Binding<Swimming>>) {
    // `PlayerBox` has lower priority, so `Dive` and `ExitWater` consume inputs first,
    // preventing `Rotate` and `EnterWater` from being triggered.
    // The consuming behavior can be configured in the `InputAction` trait.
    trigger.set_priority(1);
    trigger.bind::<Dive>().to(KeyCode::Space);
    trigger.bind::<ExitWater>().to(KeyCode::Enter);
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.entity()).unwrap();
    transform.translation += trigger.value.extend(0.0);
}

fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.entity()).unwrap();
    transform.rotate_z(FRAC_PI_4);
}

fn enter_water(
    trigger: Trigger<Started<EnterWater>>,
    mut commands: Commands,
    mut players: Query<&mut PlayerColor>,
) {
    // Change color for visibility.
    let mut color = players.get_mut(trigger.entity()).unwrap();
    **color = INDIGO_600.into();

    commands.entity(trigger.entity()).insert(Swimming);
}

fn start_diving(trigger: Trigger<Started<Dive>>, mut players: Query<&mut Visibility>) {
    let mut visibility = players.get_mut(trigger.entity()).unwrap();
    *visibility = Visibility::Hidden;
}

fn end_diving(trigger: Trigger<Completed<Dive>>, mut players: Query<&mut Visibility>) {
    let mut visibility = players.get_mut(trigger.entity()).unwrap();
    *visibility = Visibility::Visible;
}

fn exit_water(
    trigger: Trigger<Started<ExitWater>>,
    mut commands: Commands,
    mut players: Query<&mut PlayerColor>,
) {
    let mut color = players.get_mut(trigger.entity()).unwrap();
    **color = Default::default();

    commands.entity(trigger.entity()).remove::<Swimming>();
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;

/// Adds [`Swimming`].
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct EnterWater;

/// Context that overrides some actions from [`PlayerBox`].
#[derive(Component)]
struct Swimming;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Dive;

/// Removes [`Swimming`].
///
/// We set `require_reset` to `true` because [`EnterWater`] action uses the same input,
/// and we want it to be triggerable only after the button is released.
#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct ExitWater;
