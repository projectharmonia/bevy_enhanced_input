use bevy::{
    input::{
        gamepad::{GamepadAxisChangedEvent, GamepadButtonChangedEvent},
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        InputSystem,
    },
    prelude::*,
};

use super::{prelude::ActionValue, EnhancedInputSet, Input, ModKeys};

/// Adds [`InputEvent`].
///
/// Not required for [`InputContextPlugin`](super::input_context::InputContextPlugin) to function.
pub struct InputEventPlugin;

impl Plugin for InputEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>()
            .configure_sets(PreUpdate, EnhancedInputSet::SendEvents.after(InputSystem))
            .add_systems(PreUpdate, send_events.in_set(EnhancedInputSet::SendEvents));
    }
}

fn send_events(
    mut input_events: EventWriter<InputEvent>,
    mut key_events: EventReader<KeyboardInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut gamepad_button_events: EventReader<GamepadButtonChangedEvent>,
    mut gamepad_axis_events: EventReader<GamepadAxisChangedEvent>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mod_keys = ModKeys::pressed(&keys);

    let keys = key_events.read().map(move |event| InputEvent {
        entity: event.window,
        input: Input::Keyboard {
            key: event.key_code,
            // Exclude current key from modifiers.
            mod_keys: mod_keys & !ModKeys::from(event.key_code),
        },
        value: event.state.into(),
    });

    let mouse_buttons = mouse_button_events.read().map(move |event| InputEvent {
        entity: event.window,
        input: Input::MouseButton {
            button: event.button,
            mod_keys,
        },
        value: event.state.into(),
    });

    let mouse_motion = mouse_motion.read().map(move |event| InputEvent {
        entity: Entity::PLACEHOLDER,
        input: Input::MouseMotion { mod_keys },
        value: event.delta.into(),
    });

    let mouse_wheel = mouse_wheel.read().map(move |event| InputEvent {
        entity: event.window,
        input: Input::MouseWheel { mod_keys },
        value: (event.x, event.y).into(),
    });

    let gamepad_buttons = gamepad_button_events.read().map(move |event| InputEvent {
        entity: event.entity,
        input: event.button.into(),
        value: event.state.into(),
    });

    let gamepad_axes = gamepad_axis_events.read().map(|event| InputEvent {
        entity: event.entity,
        input: event.axis.into(),
        value: event.value.into(),
    });

    input_events.send_batch(
        keys.chain(mouse_buttons)
            .chain(mouse_motion)
            .chain(mouse_wheel)
            .chain(gamepad_buttons)
            .chain(gamepad_axes),
    );
}

/// An event that wraps events from `bevy_input` into a single event type.
///
/// This can be used to display appropriate button icons or bind any input sources to actions in settings.
///
/// Modifier key presses will be emitted both as separate events and included as [`ModKeys`] for other keys.
#[derive(Event, Clone, Copy, Debug, PartialEq)]
pub struct InputEvent {
    /// The entity related to the input.
    ///
    /// - For keyboard and mouse inputs, this is the window entity.
    /// - For gamepad inputs, this is the gamepad entity.
    /// - For mouse wheel inputs, this is [`Entity::PLACEHOLDER`] since winit doesn't provide this information.
    pub entity: Entity,

    /// The input that changed.
    pub input: Input,

    /// The new value of the input.
    pub value: ActionValue,
}
