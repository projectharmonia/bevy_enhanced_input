//! Demonstrates all available input conditions.
//! Press keys from the number row on the keyboard to trigger actions and observe the output in console.

use bevy::{log::LogPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    // Setup logging to display triggered events.
    let mut log_plugin = LogPlugin::default();
    log_plugin.filter += ",bevy_enhanced_input::input_action=trace";

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
        app.add_input_context::<DummyContext>()
            .add_observer(binding)
            .add_systems(Startup, spawn);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(DummyContext);
}

fn binding(mut trigger: Trigger<Binding<DummyContext>>) {
    trigger
        .bind::<PressAction>()
        .to(PressAction::KEY)
        .with_conditions(Press::default());
    trigger
        .bind::<JustPressAction>()
        .to(JustPressAction::KEY)
        .with_conditions(JustPress::default());
    trigger
        .bind::<HoldAction>()
        .to(HoldAction::KEY)
        .with_conditions(Hold::new(1.0));
    trigger
        .bind::<HoldAndReleaseAction>()
        .to(HoldAndReleaseAction::KEY)
        .with_conditions(HoldAndRelease::new(1.0));
    trigger
        .bind::<PulseAction>()
        .to(PulseAction::KEY)
        .with_conditions(Pulse::new(1.0));
    trigger
        .bind::<ReleaseAction>()
        .to(ReleaseAction::KEY)
        .with_conditions(Release::default());
    trigger
        .bind::<TapAction>()
        .to(TapAction::KEY)
        .with_conditions(Tap::new(0.5));
    trigger
        .bind::<ChordMember1>()
        .to(ChordMember1::KEY)
        .with_conditions(BlockBy::<ChordAction>::events_only()); // Don't trigger the action when the chord is active.
    trigger
        .bind::<ChordMember2>()
        .to(ChordMember2::KEY)
        .with_conditions(BlockBy::<ChordAction>::events_only());
    trigger.bind::<ChordAction>().with_conditions((
        Chord::<ChordMember1>::default(),
        Chord::<ChordMember2>::default(),
    ));
    trigger.bind::<BlockerAction>().to(BlockerAction::KEY);
    trigger
        .bind::<BlockByAction>()
        .to(BlockByAction::KEY)
        .with_conditions(BlockBy::<BlockerAction>::default());
}

#[derive(Component)]
struct DummyContext;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct PressAction;

impl PressAction {
    const KEY: KeyCode = KeyCode::Digit1;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct JustPressAction;

impl JustPressAction {
    const KEY: KeyCode = KeyCode::Digit2;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct HoldAction;

impl HoldAction {
    const KEY: KeyCode = KeyCode::Digit3;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct HoldAndReleaseAction;

impl HoldAndReleaseAction {
    const KEY: KeyCode = KeyCode::Digit4;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct PulseAction;

impl PulseAction {
    const KEY: KeyCode = KeyCode::Digit5;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ReleaseAction;

impl ReleaseAction {
    const KEY: KeyCode = KeyCode::Digit6;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct TapAction;

impl TapAction {
    const KEY: KeyCode = KeyCode::Digit7;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ChordMember1;

impl ChordMember1 {
    const KEY: KeyCode = KeyCode::Digit8;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ChordMember2;

impl ChordMember2 {
    const KEY: KeyCode = KeyCode::Digit9;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct BlockerAction;

impl BlockerAction {
    const KEY: KeyCode = KeyCode::Digit0;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ChordAction;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct BlockByAction;

impl BlockByAction {
    const KEY: KeyCode = KeyCode::Minus;
}
