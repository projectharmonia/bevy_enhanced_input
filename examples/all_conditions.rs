//! Demonstrates all available input conditions.
//! Press keys 0-9 on the keyboard to trigger actions and observe the output in console.

use bevy::{log::LogPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    // Setup logging to display triggered events.
    let mut log_plugin = LogPlugin::default();
    log_plugin.filter += ",bevy_enhanced_input::input_context::input_action=trace";

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
            .add_systems(Startup, Self::spawn);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(DummyContext);
    }
}

#[derive(Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<DummyAction>().with(DummyAction::KEY);
        ctx.bind::<BlockedByAction>()
            .with(BlockedByAction::KEY)
            .with_condition(BlockedBy::<DummyAction>::default());
        ctx.bind::<ChordAction>()
            .with(ChordAction::KEY)
            .with_condition(Chord::<DummyAction>::default());
        ctx.bind::<DownAction>()
            .with(DownAction::KEY)
            .with_condition(Down::default());
        ctx.bind::<HoldAction>()
            .with(HoldAction::KEY)
            .with_condition(Hold::new(1.0));
        ctx.bind::<HoldAndReleaseAction>()
            .with(HoldAndReleaseAction::KEY)
            .with_condition(HoldAndRelease::new(1.0));
        ctx.bind::<PressedAction>()
            .with(PressedAction::KEY)
            .with_condition(Pressed::new(1.0));
        ctx.bind::<PulseAction>()
            .with(PulseAction::KEY)
            .with_condition(Pulse::new(1.0));
        ctx.bind::<ReleasedAction>()
            .with(ReleasedAction::KEY)
            .with_condition(Released::default());
        ctx.bind::<TapAction>()
            .with(TapAction::KEY)
            .with_condition(Tap::new(0.5));

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct DummyAction;

impl DummyAction {
    const KEY: KeyCode = KeyCode::Digit0;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct BlockedByAction;

impl BlockedByAction {
    const KEY: KeyCode = KeyCode::Digit1;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct ChordAction;

impl ChordAction {
    const KEY: KeyCode = KeyCode::Digit2;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct DownAction;

impl DownAction {
    const KEY: KeyCode = KeyCode::Digit3;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct HoldAction;

impl HoldAction {
    const KEY: KeyCode = KeyCode::Digit4;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct HoldAndReleaseAction;

impl HoldAndReleaseAction {
    const KEY: KeyCode = KeyCode::Digit5;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct PressedAction;

impl PressedAction {
    const KEY: KeyCode = KeyCode::Digit6;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct PulseAction;

impl PulseAction {
    const KEY: KeyCode = KeyCode::Digit7;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct ReleasedAction;

impl ReleasedAction {
    const KEY: KeyCode = KeyCode::Digit8;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct TapAction;

impl TapAction {
    const KEY: KeyCode = KeyCode::Digit9;
}
