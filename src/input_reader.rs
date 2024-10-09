use std::marker::PhantomData;

use bevy::{
    ecs::system::SystemParam,
    input::{
        gamepad::{GamepadAxisChangedEvent, GamepadButtonInput},
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
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
pub struct InputReader<'w, 's> {
    mouse_motion_events: EventReader<'w, 's, MouseMotion>,
    mouse_wheel_events: EventReader<'w, 's, MouseWheel>,
    keyboard_events: EventReader<'w, 's, KeyboardInput>,
    mouse_button_events: EventReader<'w, 's, MouseButtonInput>,
    gamepad_button_events: EventReader<'w, 's, GamepadButtonInput>,
    gamepad_axis_events: EventReader<'w, 's, GamepadAxisChangedEvent>,
    tracker: Local<'s, InputTracker>,
}

impl InputReader<'_, '_> {
    pub fn set_ignore_keyboard(&mut self, ignore: bool) {
        self.tracker.ignore_keyboard = ignore;
    }

    pub fn set_ignore_mouse(&mut self, ignore: bool) {
        self.tracker.ignore_mouse = ignore;
    }

    pub fn read(&mut self) -> impl Iterator<Item = (Input, ActionValue)> + '_ {
        self.update_state();

        let modifiers = self.tracker.modifiers;
        let key_codes = self
            .tracker
            .key_codes
            .iter()
            .map(move |(&key_code, &value)| {
                (
                    Input::Keyboard {
                        key_code,
                        modifiers,
                    },
                    value,
                )
            });

        let mouse_buttons = self
            .tracker
            .mouse_buttons
            .iter()
            .map(move |(&button, &value)| (Input::MouseButton { button, modifiers }, value));

        let mouse_motion = self
            .tracker
            .mouse_motion
            .map(|value| (Input::MouseMotion { modifiers }, value));

        let mouse_wheel = self
            .tracker
            .mouse_wheel
            .map(|value| (Input::MouseWheel { modifiers }, value));

        let gamepad_buttons = self
            .tracker
            .gamepad_buttons
            .iter()
            .flat_map(|(&button, values)| {
                values
                    .iter()
                    .map(move |(&gamepad, &value)| (gamepad.into(), button, value))
            })
            .map(|(device, button, value)| (Input::GamepadButton { device, button }, value));

        let gamepad_axes = self
            .tracker
            .gamepad_axes
            .iter()
            .flat_map(|(&axis, values)| {
                values
                    .iter()
                    .map(move |(&gamepad, &value)| (gamepad.into(), axis, value))
            })
            .map(|(device, axis, value)| (Input::GamepadAxis { device, axis }, value));

        key_codes
            .chain(mouse_buttons)
            .chain(mouse_motion)
            .chain(mouse_wheel)
            .chain(gamepad_buttons)
            .chain(gamepad_axes)
    }

    pub(super) fn update_state(&mut self) {
        self.reset_input();

        if !self.tracker.ignore_keyboard {
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

                self.tracker
                    .key_codes
                    .insert(input.key_code, input.state.into());
            }
        }

        if !self.tracker.ignore_mouse {
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
                self.tracker
                    .mouse_buttons
                    .insert(input.button, input.state.into());
            }
        }

        for input in self.gamepad_button_events.read() {
            let buttons = self
                .tracker
                .gamepad_buttons
                .entry(input.button.button_type)
                .or_default();
            buttons.insert(input.button.gamepad, input.state.into());
        }

        for event in self.gamepad_axis_events.read() {
            let axes = self
                .tracker
                .gamepad_axes
                .entry(event.axis_type)
                .or_default();

            axes.insert(event.gamepad, event.value.into());
        }
    }

    fn reset_input(&mut self) {
        self.tracker.key_codes.clear();
        self.tracker.modifiers = KeyboardModifiers::empty();
        self.tracker.mouse_buttons.clear();
        self.tracker.mouse_motion = None;
        self.tracker.mouse_wheel = None;
        self.tracker.gamepad_buttons.clear();
        self.tracker.gamepad_axes.clear();
    }

    pub(super) fn value(&mut self, input: Input, consume: bool) -> Option<ActionValue> {
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
            Input::GamepadButton { device, button } => match device {
                GamepadDevice::Any => {
                    if consume {
                        let values = self.tracker.gamepad_buttons.remove(&button)?;

                        values
                            .iter()
                            .map(|(_, &value)| value)
                            .find(|value| value.as_bool())
                    } else {
                        let values = self.tracker.gamepad_buttons.get_mut(&button)?;

                        values
                            .iter()
                            .map(|(_, &value)| value)
                            .find(|value| value.as_bool())
                    }
                }
                GamepadDevice::Id(gamepad) => {
                    let buttons = self.tracker.gamepad_buttons.get_mut(&button)?;
                    if consume {
                        buttons.remove(&gamepad)
                    } else {
                        buttons.get(&gamepad).copied()
                    }
                }
            },
            Input::GamepadAxis { device, axis } => match device {
                GamepadDevice::Any => {
                    if consume {
                        let values = self.tracker.gamepad_axes.remove(&axis)?;
                        if values.is_empty() {
                            None
                        } else {
                            let sum: f32 = values.iter().map(|(_, value)| value.as_axis1d()).sum();
                            Some(sum.into())
                        }
                    } else {
                        let values = self.tracker.gamepad_axes.get_mut(&axis)?;
                        if values.is_empty() {
                            None
                        } else {
                            let sum: f32 = values.iter().map(|(_, value)| value.as_axis1d()).sum();
                            Some(sum.into())
                        }
                    }
                }
                GamepadDevice::Id(gamepad) => {
                    let values = self.tracker.gamepad_axes.get_mut(&axis)?;
                    if consume {
                        values.remove(&gamepad)
                    } else {
                        values.get_mut(&gamepad).copied()
                    }
                }
            },
        }
    }
}

#[derive(SystemParam)]
pub struct UiInput<'w, 's> {
    /// Marker to make the struct compile even when all features are disabled.
    marker: PhantomData<(&'w (), &'s ())>,
    #[cfg(feature = "ui_priority")]
    interactions: Query<'w, 's, &'static Interaction>,
    #[cfg(feature = "egui_priority")]
    egui: Query<'w, 's, &'static EguiContext>,
}

impl UiInput<'_, '_> {
    pub fn wants_keyboard(&self) -> bool {
        #[cfg(feature = "egui_priority")]
        if self.egui.iter().any(|ctx| ctx.get().wants_keyboard_input()) {
            return true;
        }

        false
    }

    pub fn wants_mouse(&self) -> bool {
        #[cfg(feature = "ui_priority")]
        if self
            .interactions
            .iter()
            .any(|&interaction| interaction != Interaction::None)
        {
            return true;
        }

        #[cfg(feature = "egui_priority")]
        if self
            .egui
            .iter()
            .any(|ctx| ctx.get().is_pointer_over_area() || ctx.get().wants_pointer_input())
        {
            return true;
        }

        false
    }
}

#[derive(Resource, Default)]
struct InputTracker {
    ignore_keyboard: bool,
    ignore_mouse: bool,
    key_codes: HashMap<KeyCode, ActionValue>,
    modifiers: KeyboardModifiers,
    mouse_buttons: HashMap<MouseButton, ActionValue>,
    mouse_motion: Option<ActionValue>,
    mouse_wheel: Option<ActionValue>,
    gamepad_buttons: HashMap<GamepadButtonType, HashMap<Gamepad, ActionValue>>,
    gamepad_axes: HashMap<GamepadAxisType, HashMap<Gamepad, ActionValue>>,
}

bitflags! {
    /// Modifiers for both left and right keys.
    #[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

/// All input that can be associated with an action.
///
/// See also [`InputReader::read`] for binding input at runtime.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
    GamepadButton {
        device: GamepadDevice,
        button: GamepadButtonType,
    },
    GamepadAxis {
        device: GamepadDevice,
        axis: GamepadAxisType,
    },
}

impl Input {
    /// Returns [`Input::MouseMotion`] without keyboard modifiers.
    pub fn mouse_motion() -> Self {
        Self::MouseMotion {
            modifiers: KeyboardModifiers::empty(),
        }
    }

    /// Returns [`Input::MouseWheel`] without keyboard modifiers.
    pub fn mouse_wheel() -> Self {
        Self::MouseWheel {
            modifiers: KeyboardModifiers::empty(),
        }
    }
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

/// Associated gamepad for [`Input`].
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum GamepadDevice {
    /// Matches input from any gamepad.
    ///
    /// For an axis, the [`ActionValue`] will be calculated as the sum of inputs from all gamepads.
    /// For a button, the [`ActionValue`] will be `true` if any gamepad has this button pressed.
    Any,

    /// Matches input from specific gamepad.
    Id(Gamepad),
}

impl From<Gamepad> for GamepadDevice {
    fn from(value: Gamepad) -> Self {
        Self::Id(value)
    }
}
