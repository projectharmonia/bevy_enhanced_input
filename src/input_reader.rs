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

/// Reads input from multiple sources.
///
/// We use event-based reading to prevent newly created
/// context access previosly pressed inputs.
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
    egui: Query<'w, 's, &'static EguiContext>,
}

impl InputReader<'_, '_> {
    /// Reads all events and transforms into [`Input`] representation.
    ///
    /// Should be called on each system run before [`Self::value`].
    pub(super) fn update_state(&mut self) {
        self.reset_input();

        if !self.ui_wants_keyboard() {
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

        if !self.ui_wants_mouse() {
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

    fn ui_wants_keyboard(&self) -> bool {
        #[cfg(feature = "egui_priority")]
        if self.egui.iter().any(|ctx| ctx.get().wants_keyboard_input()) {
            return true;
        }

        false
    }

    fn ui_wants_mouse(&self) -> bool {
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

    /// Returns the [`ActionValue`] for the given [`Input`] from input events.
    ///
    /// Returns [`None`] if there were no events for the given input.
    ///
    /// For gamepad input, it exclusively reads from the specified `gamepad`.
    /// If `consume` is `true`, the value will be consumed and unavailable for subsequent calls.
    pub(super) fn value(
        &mut self,
        input: Input,
        gamepad: GamepadDevice,
        consume: bool,
    ) -> Option<ActionValue> {
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
            Input::GamepadButton { button } => match gamepad {
                GamepadDevice::Any => {
                    if consume {
                        let values = self.tracker.gamepad_buttons.remove(&button)?;

                        values
                            .iter()
                            .map(|(_, &value)| value)
                            .max_by_key(|value| value.as_bool())
                    } else {
                        let values = self.tracker.gamepad_buttons.get_mut(&button)?;

                        values
                            .iter()
                            .map(|(_, &value)| value)
                            .max_by_key(|value| value.as_bool())
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
            Input::GamepadAxis { axis } => match gamepad {
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

/// Accumulates informations from events.
#[derive(Resource, Default)]
struct InputTracker {
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

/// Inputs that can be associated with an
/// [`InputAction`](super::input_context::input_action::InputAction).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Input {
    /// Keyboard button, will be captured as [`ActionValue::Bool`].
    Keyboard {
        key_code: KeyCode,
        modifiers: KeyboardModifiers,
    },
    /// Mouse button, will be captured as [`ActionValue::Bool`].
    MouseButton {
        button: MouseButton,
        modifiers: KeyboardModifiers,
    },
    /// Mouse movement, will be captured as [`ActionValue::Axis2D`].
    MouseMotion { modifiers: KeyboardModifiers },
    /// Mouse wheel, will be captured as [`ActionValue::Axis1D`].
    MouseWheel { modifiers: KeyboardModifiers },
    /// Gamepad button, will be captured as [`ActionValue::Bool`].
    GamepadButton { button: GamepadButtonType },
    /// Gamepad stick axis, will be captured as [`ActionValue::Axis1D`].
    GamepadAxis { axis: GamepadAxisType },
}

impl Input {
    /// Returns [`Input::MouseMotion`] without keyboard modifiers.
    pub fn mouse_motion() -> Self {
        Self::MouseMotion {
            modifiers: Default::default(),
        }
    }

    /// Returns [`Input::MouseWheel`] without keyboard modifiers.
    pub fn mouse_wheel() -> Self {
        Self::MouseWheel {
            modifiers: Default::default(),
        }
    }
}

impl From<KeyCode> for Input {
    fn from(key_code: KeyCode) -> Self {
        Self::Keyboard {
            key_code,
            modifiers: Default::default(),
        }
    }
}

impl From<MouseButton> for Input {
    fn from(button: MouseButton) -> Self {
        Self::MouseButton {
            button,
            modifiers: Default::default(),
        }
    }
}

impl From<GamepadButtonType> for Input {
    fn from(button: GamepadButtonType) -> Self {
        Self::GamepadButton { button }
    }
}

impl From<GamepadAxisType> for Input {
    fn from(axis: GamepadAxisType) -> Self {
        Self::GamepadAxis { axis }
    }
}

/// Associated gamepad.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum GamepadDevice {
    /// Matches input from any gamepad.
    ///
    /// For an axis, the [`ActionValue`] will be calculated as the sum of inputs from all gamepads.
    /// For a button, the [`ActionValue`] will be `true` if any gamepad has this button pressed.
    #[default]
    Any,

    /// Matches input from specific gamepad.
    Id(Gamepad),
}

impl From<Gamepad> for GamepadDevice {
    fn from(value: Gamepad) -> Self {
        Self::Id(value)
    }
}

impl From<usize> for GamepadDevice {
    fn from(value: usize) -> Self {
        Gamepad::new(value).into()
    }
}
