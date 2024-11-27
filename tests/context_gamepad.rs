use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadInfo},
        InputPlugin,
    },
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

#[test]
fn any() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<AnyGamepad>();

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

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<AnyGamepad>(entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let other_button = GamepadButton {
        gamepad: other_gamepad,
        button_type: DummyAction::BUTTON,
    };

    let mut buttons = app.world_mut().resource_mut::<ButtonInput<GamepadButton>>();
    buttons.release(button);
    buttons.press(other_button);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<AnyGamepad>(entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);
}

#[test]
fn by_id() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<FirstGamepad>();

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

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<FirstGamepad>(entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let other_button = GamepadButton {
        gamepad: other_gamepad,
        button_type: DummyAction::BUTTON,
    };

    let mut buttons = app.world_mut().resource_mut::<ButtonInput<GamepadButton>>();
    buttons.release(button);
    buttons.press(other_button);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<FirstGamepad>(entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::None);
}

#[derive(Debug, Component, InputContext)]
#[input_context(instance_system = any_gamepad_instance)]
struct AnyGamepad;

fn any_gamepad_instance(In(_): In<Entity>) -> ContextInstance {
    let mut ctx = ContextInstance::with_gamepad(GamepadDevice::Any);
    ctx.bind::<DummyAction>().with(DummyAction::BUTTON);
    ctx
}

#[derive(Debug, Component, InputContext)]
#[input_context(instance_system = first_gamepad_instance)]
struct FirstGamepad;

fn first_gamepad_instance(In(_): In<Entity>) -> ContextInstance {
    let mut ctx = ContextInstance::with_gamepad(0);
    ctx.bind::<DummyAction>().with(DummyAction::BUTTON);
    ctx
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const BUTTON: GamepadButtonType = GamepadButtonType::South;
}
