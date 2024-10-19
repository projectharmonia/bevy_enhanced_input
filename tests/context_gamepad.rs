mod action_recorder;

use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadInfo},
        InputPlugin,
    },
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn any() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<AnyGamepad>()
    .record_action::<DummyAction>();

    let gamepad = Gamepad::new(0);
    app.world_mut().send_event(GamepadConnectionEvent {
        gamepad,
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "Dummy 1".to_string(),
        }),
    });

    let other_gamepad = Gamepad::new(1);
    app.world_mut().send_event(GamepadConnectionEvent {
        gamepad: other_gamepad,
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "Dummy 2".to_string(),
        }),
    });

    let entity = app.world_mut().spawn(AnyGamepad).id();

    app.update();

    let button = GamepadButton {
        gamepad,
        button_type: DummyAction::BUTTON,
    };

    app.world_mut()
        .resource_mut::<ButtonInput<GamepadButton>>()
        .press(button);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<DummyAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let other_button = GamepadButton {
        gamepad: other_gamepad,
        button_type: DummyAction::BUTTON,
    };

    let mut buttons = app.world_mut().resource_mut::<ButtonInput<GamepadButton>>();
    buttons.release(button);
    buttons.press(other_button);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<DummyAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);
}

#[test]
fn by_id() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<FirstGamepad>()
    .record_action::<DummyAction>();

    let gamepad = Gamepad::new(0);
    app.world_mut().send_event(GamepadConnectionEvent {
        gamepad,
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "Dummy 1".to_string(),
        }),
    });

    let other_gamepad = Gamepad::new(1);
    app.world_mut().send_event(GamepadConnectionEvent {
        gamepad: other_gamepad,
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "Dummy 2".to_string(),
        }),
    });

    let entity = app.world_mut().spawn(FirstGamepad).id();

    app.update();

    let button = GamepadButton {
        gamepad,
        button_type: DummyAction::BUTTON,
    };

    app.world_mut()
        .resource_mut::<ButtonInput<GamepadButton>>()
        .press(button);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<DummyAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let other_button = GamepadButton {
        gamepad: other_gamepad,
        button_type: DummyAction::BUTTON,
    };

    let mut buttons = app.world_mut().resource_mut::<ButtonInput<GamepadButton>>();
    buttons.release(button);
    buttons.press(other_button);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<DummyAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::None);
}

#[derive(Debug, Component)]
struct AnyGamepad;

impl InputContext for AnyGamepad {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::with_gamepad(GamepadDevice::Any);
        ctx.bind::<DummyAction>().with(DummyAction::BUTTON);
        ctx
    }
}

#[derive(Debug, Component)]
struct FirstGamepad;

impl InputContext for FirstGamepad {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::with_gamepad(0);
        ctx.bind::<DummyAction>().with(DummyAction::BUTTON);
        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct DummyAction;

impl DummyAction {
    const BUTTON: GamepadButtonType = GamepadButtonType::South;
}
