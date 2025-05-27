use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn keys() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

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
        (KeyCode::Digit0, FORWARD),
        (KeyCode::Digit1, BACKWARD),
        (KeyCode::Digit2, LEFT),
        (KeyCode::Digit3, RIGHT),
        (KeyCode::Digit4, UP),
        (KeyCode::Digit5, DOWN),
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);

        app.update();

        let actions = app.world().get::<Actions<Test>>(entity).unwrap();
        assert_eq!(
            actions.value::<TestAction>().unwrap(),
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
        .add_input_context::<Test>()
        .add_observer(binding)
        .finish();

    let gamepad_entity = app.world_mut().spawn(Gamepad::default()).id();
    let ctx_entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    for (button, dir) in [
        (GamepadButton::DPadUp, UP),
        (GamepadButton::DPadLeft, LEFT),
        (GamepadButton::DPadDown, DOWN),
        (GamepadButton::DPadRight, RIGHT),
    ] {
        let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
        gamepad.analog_mut().set(button, 1.0);

        app.update();

        let actions = app.world().get::<Actions<Test>>(ctx_entity).unwrap();
        assert_eq!(
            actions.value::<TestAction>().unwrap(),
            dir.into(),
            "`{button:?}` should result in `{dir}`"
        );

        let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
        gamepad.analog_mut().set(button, 0.0);

        app.update();
    }
}

#[test]
fn sticks() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(binding)
        .finish();

    let gamepad_entity = app.world_mut().spawn(Gamepad::default()).id();
    let ctx_entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    for (axis, dirs) in [
        (GamepadAxis::LeftStickX, [LEFT, RIGHT]),
        (GamepadAxis::RightStickX, [LEFT, RIGHT]),
        (GamepadAxis::LeftStickY, [DOWN, UP]),
        (GamepadAxis::RightStickY, [DOWN, UP]),
    ] {
        for (dir, value) in dirs.into_iter().zip([-1.0, 1.0]) {
            let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
            gamepad.analog_mut().set(axis, value);

            app.update();

            let actions = app.world().get::<Actions<Test>>(ctx_entity).unwrap();
            assert_eq!(
                actions.value::<TestAction>().unwrap(),
                dir.into(),
                "`{axis:?}` should result in `{dir}`"
            );

            let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
            gamepad.analog_mut().set(axis, 0.0);

            app.update();
        }
    }
}

const RIGHT: Vec3 = Vec3::X;
const LEFT: Vec3 = Vec3::NEG_X;
const BACKWARD: Vec3 = Vec3::Z;
const FORWARD: Vec3 = Vec3::NEG_Z;
const UP: Vec3 = Vec3::Y;
const DOWN: Vec3 = Vec3::NEG_Y;

fn binding(trigger: Trigger<Binding<Test>>, mut actions: Query<&mut Actions<Test>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<TestAction>().to((
        Cardinal::wasd_keys(),
        Cardinal::arrow_keys(),
        Cardinal::dpad_buttons(),
        Bidirectional {
            positive: KeyCode::NumpadAdd,
            negative: KeyCode::NumpadSubtract,
        },
        Axial::left_stick(),
        Axial::right_stick(),
        Spatial {
            forward: KeyCode::Digit0,
            backward: KeyCode::Digit1,
            left: KeyCode::Digit2,
            right: KeyCode::Digit3,
            up: KeyCode::Digit4,
            down: KeyCode::Digit5,
        },
    ));
}

#[derive(InputContext)]
struct Test;

#[derive(Debug, InputAction)]
#[input_action(output = Vec3, consume_input = true)]
struct TestAction;
