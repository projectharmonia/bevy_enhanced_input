use bevy::{
    ecs::system::SystemParam,
    input::{
        gamepad::{GamepadAxisChangedEvent, GamepadButtonInput},
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        ButtonState,
    },
    prelude::*,
    utils::HashMap,
};
#[cfg(feature = "egui_priority")]
use bevy_egui::EguiContext;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::action_value::ActionValue;

#[derive(SystemParam)]
pub(super) struct InputReader<'w, 's> {
    mouse_motion_events: EventReader<'w, 's, MouseMotion>,
    mouse_wheel_events: EventReader<'w, 's, MouseWheel>,
    keyboard_events: EventReader<'w, 's, KeyboardInput>,
    mouse_button_events: EventReader<'w, 's, MouseButtonInput>,
    gamepad_button_events: EventReader<'w, 's, GamepadButtonInput>,
    gamepad_axis_events: EventReader<'w, 's, GamepadAxisChangedEvent>,
    tracker: Local<'s, InputTracker>,
    #[cfg(feature = "ui_priority")]
    interactions: Query<'w, 's, &'static Interaction>,
    #[cfg(feature = "egui_priority")]
    egui: Query<'w, 's, &'static mut EguiContext>,
}

impl InputReader<'_, '_> {
    pub(super) fn update_state(&mut self) {
        for input in self.keyboard_events.read() {
            // Record modifiers redundantly for quick access.
            match input.key_code {
                KeyCode::AltLeft | KeyCode::AltRight => {
                    self.tracker.modifiers &= KeyboardModifiers::ALT;
                }
                KeyCode::ControlLeft | KeyCode::ControlRight => {
                    self.tracker.modifiers &= KeyboardModifiers::CONTROL;
                }
                KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                    self.tracker.modifiers &= KeyboardModifiers::SHIFT;
                }
                KeyCode::SuperLeft | KeyCode::SuperRight => {
                    self.tracker.modifiers &= KeyboardModifiers::SUPER;
                }
                _ => (),
            }

            let pressed = match input.state {
                ButtonState::Pressed => true.into(),
                ButtonState::Released => false.into(),
            };

            self.tracker.key_codes.insert(input.key_code, pressed);
        }

        if !self.mouse_motion_events.is_empty() {
            let mouse_motion: Vec2 = self
                .mouse_motion_events
                .read()
                .map(|event| event.delta)
                .sum();
            self.tracker.mouse_motion = Some(mouse_motion.into());
        }

        if !self.mouse_wheel_events.is_empty() {
            let mouse_wheel: Vec2 = self
                .mouse_wheel_events
                .read()
                .map(|event| Vec2::new(event.x, event.y))
                .sum();
            self.tracker.mouse_wheel = Some(mouse_wheel.into());
        }

        for input in self.mouse_button_events.read() {
            let pressed = match input.state {
                ButtonState::Pressed => true.into(),
                ButtonState::Released => false.into(),
            };

            self.tracker.mouse_buttons.insert(input.button, pressed);
        }

        for input in self.gamepad_button_events.read() {
            let pressed = match input.state {
                ButtonState::Pressed => true.into(),
                ButtonState::Released => false.into(),
            };

            self.tracker.gamepad_buttons.insert(input.button, pressed);
        }

        for event in self.gamepad_axis_events.read() {
            let axis = GamepadAxis {
                gamepad: event.gamepad,
                axis_type: event.axis_type,
            };

            self.tracker.gamepad_axes.insert(axis, event.value.into());
        }

        #[cfg(feature = "ui_priority")]
        {
            if self
                .interactions
                .iter()
                .any(|&interaction| interaction != Interaction::None)
            {
                self.tracker.mouse_buttons.clear();
                self.tracker.mouse_wheel = None;
            }
        }

        #[cfg(feature = "egui_priority")]
        {
            if self.egui.iter_mut().any(|mut ctx| {
                ctx.get_mut().is_pointer_over_area() || ctx.get_mut().wants_pointer_input()
            }) {
                self.tracker.mouse_buttons.clear();
                self.tracker.mouse_wheel = None;
            }

            if self
                .egui
                .iter_mut()
                .any(|mut ctx| ctx.get_mut().wants_keyboard_input())
            {
                self.tracker.key_codes.clear();
                self.tracker.modifiers = KeyboardModifiers::empty();
            }
        }
    }

    pub(super) fn read(&mut self, input: Input, consume: bool) -> Option<ActionValue> {
        match input {
            Input::Keyboard {
                key_code,
                modifiers,
            } => {
                if !self.tracker.modifiers.contains(modifiers) {
                    return None;
                }

                if consume {
                    self.tracker.key_codes.remove(&key_code)
                } else {
                    self.tracker.key_codes.get(&key_code).copied()
                }
            }
            Input::MouseButton { button, modifiers } => {
                if !self.tracker.modifiers.contains(modifiers) {
                    return None;
                }

                if consume {
                    self.tracker.mouse_buttons.remove(&button)
                } else {
                    self.tracker.mouse_buttons.get(&button).copied()
                }
            }
            Input::MouseMotion { modifiers } => {
                if !self.tracker.modifiers.contains(modifiers) {
                    return None;
                }

                if consume {
                    self.tracker.mouse_motion.take()
                } else {
                    self.tracker.mouse_motion
                }
            }
            Input::MouseWheel { modifiers } => {
                if !self.tracker.modifiers.contains(modifiers) {
                    return None;
                }

                if consume {
                    self.tracker.mouse_wheel.take()
                } else {
                    self.tracker.mouse_wheel
                }
            }
            Input::GamepadButton(gamepad_button) => {
                if consume {
                    self.tracker.gamepad_buttons.remove(&gamepad_button)
                } else {
                    self.tracker.gamepad_buttons.get(&gamepad_button).copied()
                }
            }
            Input::GamepadAxis(gamepad_axis) => {
                if consume {
                    self.tracker.gamepad_axes.remove(&gamepad_axis)
                } else {
                    self.tracker.gamepad_axes.get(&gamepad_axis).copied()
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct InputTracker {
    key_codes: HashMap<KeyCode, ActionValue>,
    modifiers: KeyboardModifiers,
    mouse_motion: Option<ActionValue>,
    mouse_wheel: Option<ActionValue>,
    mouse_buttons: HashMap<MouseButton, ActionValue>,
    gamepad_buttons: HashMap<GamepadButton, ActionValue>,
    gamepad_axes: HashMap<GamepadAxis, ActionValue>,
}

bitflags! {
    /// Modifiers for both left and right keys.
    #[derive(Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    pub struct KeyboardModifiers: u8 {
        /// Corresponds to [`KeyCode::AltLeft`] and [`KeyCode::AltRight`].
        const ALT = 0b00000001;
        /// Corresponds to [`KeyCode::ControlLeft`] and [`KeyCode::ControlRight`].
        const CONTROL = 0b00000010;
        /// Corresponds to [`KeyCode::ShiftLeft`] and [`KeyCode::ShiftRight`]
        const SHIFT = 0b00000100;
        /// Corresponds to [`KeyCode::SuperLeft`] and [`KeyCode::SuperRight`].
        const SUPER = 0b00001000;
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Input {
    Keyboard {
        key_code: KeyCode,
        modifiers: KeyboardModifiers,
    },
    MouseButton {
        button: MouseButton,
        modifiers: KeyboardModifiers,
    },
    MouseMotion {
        modifiers: KeyboardModifiers,
    },
    MouseWheel {
        modifiers: KeyboardModifiers,
    },
    GamepadButton(GamepadButton),
    GamepadAxis(GamepadAxis),
}

impl From<KeyCode> for Input {
    fn from(key_code: KeyCode) -> Self {
        Self::Keyboard {
            key_code,
            modifiers: KeyboardModifiers::empty(),
        }
    }
}

impl From<MouseButton> for Input {
    fn from(button: MouseButton) -> Self {
        Self::MouseButton {
            button,
            modifiers: KeyboardModifiers::empty(),
        }
    }
}
