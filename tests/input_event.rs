use bevy::{
    input::{
        gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent},
        keyboard::{Key, KeyboardInput},
        mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
        ButtonState, InputPlugin,
    },
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

#[test]
fn keyboard() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins));

    app.world_mut().send_event(KeyboardInput {
        key_code: KeyCode::ControlLeft,
        logical_key: Key::Control,
        state: ButtonState::Pressed,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.world_mut().send_event(KeyboardInput {
        key_code: KeyCode::KeyA,
        logical_key: Key::Dead(None),
        state: ButtonState::Pressed,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });

    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<InputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        [
            InputEvent {
                entity: Entity::PLACEHOLDER,
                input: Input::Keyboard {
                    key: KeyCode::ControlLeft,
                    mod_keys: Default::default()
                },
                value: true.into()
            },
            InputEvent {
                entity: Entity::PLACEHOLDER,
                input: Input::Keyboard {
                    key: KeyCode::KeyA,
                    mod_keys: ModKeys::CONTROL
                },
                value: true.into()
            }
        ]
    );
}

#[test]
fn mouse_button() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins));

    app.world_mut().send_event(MouseButtonInput {
        button: MouseButton::Left,
        state: ButtonState::Pressed,
        window: Entity::PLACEHOLDER,
    });

    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<InputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        [InputEvent {
            entity: Entity::PLACEHOLDER,
            input: MouseButton::Left.into(),
            value: true.into()
        }]
    );
}

#[test]
fn mouse_motion() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins));

    app.world_mut().send_event(MouseMotion { delta: Vec2::ONE });

    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<InputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        [InputEvent {
            entity: Entity::PLACEHOLDER,
            input: Input::mouse_motion(),
            value: Vec2::ONE.into()
        }]
    );
}

#[test]
fn mouse_wheel() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins));

    app.world_mut().send_event(MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 2.0,
        y: 2.0,
        window: Entity::PLACEHOLDER,
    });

    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<InputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        [InputEvent {
            entity: Entity::PLACEHOLDER,
            input: Input::mouse_wheel(),
            value: Vec2::new(2.0, 2.0).into()
        }]
    );
}

#[test]
fn gamepad_button() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins));

    app.world_mut().send_event(GamepadButtonChangedEvent {
        entity: Entity::PLACEHOLDER,
        button: GamepadButton::North,
        state: ButtonState::Pressed,
        value: 1.0,
    });

    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<InputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        [InputEvent {
            entity: Entity::PLACEHOLDER,
            input: Input::GamepadButton(GamepadButton::North),
            value: true.into()
        }]
    );
}

#[test]
fn gamepad_axis() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins));

    app.world_mut().send_event(GamepadAxisChangedEvent {
        entity: Entity::PLACEHOLDER,
        axis: GamepadAxis::LeftStickX,
        value: 1.0,
    });

    app.update();

    let events: Vec<_> = app
        .world_mut()
        .resource_mut::<Events<InputEvent>>()
        .drain()
        .collect();
    assert_eq!(
        events,
        [InputEvent {
            entity: Entity::PLACEHOLDER,
            input: Input::GamepadAxis(GamepadAxis::LeftStickX),
            value: 1.0.into()
        }]
    );
}
