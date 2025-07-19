//! Demonstrates all available input conditions.
//! Press keys from the number row on the keyboard to trigger actions and observe the output in console.

use bevy::{ecs::spawn::SpawnWith, log::LogPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    // Setup logging to display triggered events.
    let mut log_plugin = LogPlugin::default();
    log_plugin.filter += ",bevy_enhanced_input=debug";

    App::new()
        .add_plugins((
            DefaultPlugins.set(log_plugin),
            EnhancedInputPlugin,
            GamePlugin,
        ))
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<TestContext>()
            .add_systems(Startup, spawn);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            context.spawn((
                Action::<PressAction>::new(),
                Down::default(),
                bindings![PressAction::KEY],
            ));
            context.spawn((
                Action::<JustPressAction>::new(),
                Press::default(),
                bindings![JustPressAction::KEY],
            ));
            context.spawn((
                Action::<HoldAction>::new(),
                Hold::new(1.0),
                bindings![HoldAction::KEY],
            ));
            context.spawn((
                Action::<HoldAndReleaseAction>::new(),
                HoldAndRelease::new(1.0),
                bindings![HoldAndReleaseAction::KEY],
            ));
            context.spawn((
                Action::<PulseAction>::new(),
                Pulse::new(1.0),
                bindings![PulseAction::KEY],
            ));
            context.spawn((
                Action::<ReleaseAction>::new(),
                Release::default(),
                bindings![ReleaseAction::KEY],
            ));
            context.spawn((
                Action::<TapAction>::new(),
                Tap::new(0.5),
                bindings![TapAction::KEY],
            ));

            let chord1 = context
                .spawn((Action::<ChordMember1>::new(), bindings![ChordMember1::KEY]))
                .id();
            let chord2 = context
                .spawn((Action::<ChordMember2>::new(), bindings![ChordMember2::KEY]))
                .id();

            context.spawn((Action::<ChordAction>::new(), Chord::new([chord1, chord2])));

            let blocker = context
                .spawn((
                    Action::<BlockerAction>::new(),
                    bindings![BlockerAction::KEY],
                ))
                .id();
            context.spawn((
                Action::<BlockByAction>::new(),
                BlockBy::single(blocker),
                bindings![BlockByAction::KEY],
            ));
        })),
    ));
}

#[derive(Component, InputContext)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct PressAction;

impl PressAction {
    const KEY: KeyCode = KeyCode::Digit1;
}

#[derive(InputAction)]
#[action_output(bool)]
struct JustPressAction;

impl JustPressAction {
    const KEY: KeyCode = KeyCode::Digit2;
}

#[derive(InputAction)]
#[action_output(bool)]
struct HoldAction;

impl HoldAction {
    const KEY: KeyCode = KeyCode::Digit3;
}

#[derive(InputAction)]
#[action_output(bool)]
struct HoldAndReleaseAction;

impl HoldAndReleaseAction {
    const KEY: KeyCode = KeyCode::Digit4;
}

#[derive(InputAction)]
#[action_output(bool)]
struct PulseAction;

impl PulseAction {
    const KEY: KeyCode = KeyCode::Digit5;
}

#[derive(InputAction)]
#[action_output(bool)]
struct ReleaseAction;

impl ReleaseAction {
    const KEY: KeyCode = KeyCode::Digit6;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TapAction;

impl TapAction {
    const KEY: KeyCode = KeyCode::Digit7;
}

#[derive(InputAction)]
#[action_output(bool)]
struct ChordMember1;

impl ChordMember1 {
    const KEY: KeyCode = KeyCode::Digit8;
}

#[derive(InputAction)]
#[action_output(bool)]
struct ChordMember2;

impl ChordMember2 {
    const KEY: KeyCode = KeyCode::Digit9;
}

#[derive(InputAction)]
#[action_output(bool)]
struct BlockerAction;

impl BlockerAction {
    const KEY: KeyCode = KeyCode::Digit0;
}

#[derive(InputAction)]
#[action_output(bool)]
struct ChordAction;

#[derive(InputAction)]
#[action_output(bool)]
struct BlockByAction;

impl BlockByAction {
    const KEY: KeyCode = KeyCode::Minus;
}
