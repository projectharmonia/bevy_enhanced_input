use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadInfo},
        InputPlugin,
    },
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

#[test]
fn keys() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    for (key, dir) in [
        (KeyCode::KeyW, UP),
        (KeyCode::KeyA, LEFT),
        (KeyCode::KeyS, DOWN),
        (KeyCode::KeyD, RIGHT),
        (KeyCode::ArrowUp, UP),
        (KeyCode::ArrowLeft, LEFT),
        (KeyCode::ArrowDown, DOWN),
        (KeyCode::ArrowRight, RIGHT),
        (KeyCode::NumpadSubtract, LEFT),
        (KeyCode::NumpadAdd, RIGHT),
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);

        app.update();

        let instances = app.world().resource::<ContextInstances>();
        let ctx = instances.get::<DummyContext>(entity).unwrap();
        let action = ctx.action::<DummyAction>().unwrap();
        assert_eq!(
            action.value(),
            dir.into(),
            "`{key:?}` should result in `{dir}`"
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(key);

        app.update();
    }
}

#[test]
fn dpad() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let gamepad = Gamepad::new(0);
    app.world_mut().send_event(GamepadConnectionEvent {
        gamepad,
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "Dummy".to_string(),
        }),
    });

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    for (button, dir) in [
        (GamepadButtonType::DPadUp, UP),
        (GamepadButtonType::DPadLeft, LEFT),
        (GamepadButtonType::DPadDown, DOWN),
        (GamepadButtonType::DPadRight, RIGHT),
    ] {
        let button = GamepadButton {
            gamepad,
            button_type: button,
        };

        app.world_mut()
            .resource_mut::<ButtonInput<GamepadButton>>()
            .press(button);

        app.update();

        let instances = app.world().resource::<ContextInstances>();
        let ctx = instances.get::<DummyContext>(entity).unwrap();
        let action = ctx.action::<DummyAction>().unwrap();
        assert_eq!(
            action.value(),
            dir.into(),
            "`{button:?}` should result in `{dir}`"
        );

        app.world_mut()
            .resource_mut::<ButtonInput<GamepadButton>>()
            .release(button);

        app.update();
    }
}

#[test]
fn sticks() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let gamepad = Gamepad::new(0);
    app.world_mut().send_event(GamepadConnectionEvent {
        gamepad,
        connection: GamepadConnection::Connected(GamepadInfo {
            name: "Dummy".to_string(),
        }),
    });

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    for (axis, dirs) in [
        (GamepadAxisType::LeftStickX, [LEFT, RIGHT]),
        (GamepadAxisType::RightStickX, [LEFT, RIGHT]),
        (GamepadAxisType::LeftStickY, [DOWN, UP]),
        (GamepadAxisType::RightStickY, [DOWN, UP]),
    ] {
        let axis = GamepadAxis {
            gamepad,
            axis_type: axis,
        };

        for (dir, value) in dirs.into_iter().zip([-1.0, 1.0]) {
            app.world_mut()
                .resource_mut::<Axis<GamepadAxis>>()
                .set(axis, value);

            app.update();

            let instances = app.world().resource::<ContextInstances>();
            let ctx = instances.get::<DummyContext>(entity).unwrap();
            let action = ctx.action::<DummyAction>().unwrap();
            assert_eq!(
                action.value(),
                dir.into(),
                "`{axis:?}` should result in `{dir}`"
            );

            app.world_mut()
                .resource_mut::<Axis<GamepadAxis>>()
                .set(axis, 0.0);

            app.update();
        }
    }
}

const UP: Vec2 = Vec2::new(0.0, 1.0);
const LEFT: Vec2 = Vec2::new(-1.0, 0.0);
const DOWN: Vec2 = Vec2::new(0.0, -1.0);
const RIGHT: Vec2 = Vec2::new(1.0, 0.0);

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<DummyAction>().to((
            Cardinal::wasd_keys(),
            Cardinal::arrow_keys(),
            Cardinal::dpad_buttons(),
            Biderectional {
                positive: KeyCode::NumpadAdd,
                negative: KeyCode::NumpadSubtract,
            },
            GamepadStick::Left,
            GamepadStick::Right,
        ));

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, consume_input = true)]
struct DummyAction;
