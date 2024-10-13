use bevy::{
    ecs::system::SystemParam,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    utils::HashSet,
};
#[cfg(feature = "egui_priority")]
use bevy_egui::EguiContext;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::action_value::ActionValue;

/// Reads input from multiple sources.
#[derive(SystemParam)]
pub(super) struct InputReader<'w, 's> {
    key_codes: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    mouse_motion_events: EventReader<'w, 's, MouseMotion>,
    mouse_wheel_events: EventReader<'w, 's, MouseWheel>,
    gamepad_buttons: Res<'w, ButtonInput<GamepadButton>>,
    gamepad_axis: Res<'w, Axis<GamepadAxis>>,
    gamepads: Res<'w, Gamepads>,
    consumed: Local<'s, ConsumedInput>,
    #[cfg(feature = "ui_priority")]
    interactions: Query<'w, 's, &'static Interaction>,
    #[cfg(feature = "egui_priority")]
    egui: Query<'w, 's, &'static EguiContext>,
}

impl InputReader<'_, '_> {
    /// Resets all consumed values.
    pub(super) fn reset(&mut self) {
        self.consumed.reset();

        #[cfg(feature = "egui_priority")]
        if self.egui.iter().any(|ctx| ctx.get().wants_keyboard_input()) {
            self.consumed.ui_wants_keyboard = true;
        }

        #[cfg(feature = "ui_priority")]
        if self
            .interactions
            .iter()
            .any(|&interaction| interaction != Interaction::None)
        {
            self.consumed.ui_wants_mouse = true;
        }

        #[cfg(feature = "egui_priority")]
        if self
            .egui
            .iter()
            .any(|ctx| ctx.get().is_pointer_over_area() || ctx.get().wants_pointer_input())
        {
            self.consumed.ui_wants_mouse = true;
        }
    }

    /// Returns the [`ActionValue`] for the given [`Input`] if exists.
    ///
    /// For gamepad input, it exclusively reads from the specified `gamepad`.
    /// If `consume` is `true`, the value will be consumed and unavailable for subsequent calls.
    pub(super) fn value(
        &mut self,
        input: Input,
        gamepad: GamepadDevice,
        consume: bool,
    ) -> ActionValue {
        match input {
            Input::Keyboard {
                key_code,
                modifiers,
            } => {
                let pressed = self.key_codes.pressed(key_code)
                    && !self.consumed.key_codes.contains(&key_code)
                    && self.modifiers_pressed(modifiers);

                if pressed && consume {
                    self.consumed.key_codes.insert(key_code);
                    self.consumed.modifiers.insert(modifiers);
                }

                pressed.into()
            }
            Input::MouseButton { button, modifiers } => {
                let pressed = self.mouse_buttons.pressed(button)
                    && !self.consumed.mouse_buttons.contains(&button)
                    && self.modifiers_pressed(modifiers);

                if pressed && consume {
                    self.consumed.mouse_buttons.insert(button);
                    self.consumed.modifiers.insert(modifiers);
                }

                pressed.into()
            }
            Input::MouseMotion { modifiers } => {
                if self.consumed.mouse_motion || !self.modifiers_pressed(modifiers) {
                    return Vec2::ZERO.into();
                }

                let value: Vec2 = self
                    .mouse_motion_events
                    .read()
                    .map(|event| event.delta)
                    .sum();

                self.consumed.mouse_motion = true;

                value.into()
            }
            Input::MouseWheel { modifiers } => {
                if self.consumed.mouse_wheel || !self.modifiers_pressed(modifiers) {
                    return Vec2::ZERO.into();
                }

                let value: Vec2 = self
                    .mouse_wheel_events
                    .read()
                    .map(|event| Vec2::new(event.x, event.y))
                    .sum();

                self.consumed.mouse_wheel = true;

                value.into()
            }
            Input::GamepadButton { button } => {
                if self.consumed.gamepad_buttons.contains(&(gamepad, button)) {
                    return false.into();
                }

                let pressed = match gamepad {
                    GamepadDevice::Any => self.gamepads.iter().any(|gamepad| {
                        self.gamepad_buttons.pressed(GamepadButton {
                            gamepad,
                            button_type: button,
                        })
                    }),
                    GamepadDevice::Id(gamepad) => self.gamepad_buttons.pressed(GamepadButton {
                        gamepad,
                        button_type: button,
                    }),
                };

                if pressed && consume {
                    self.consumed.gamepad_buttons.insert((gamepad, button));
                }

                pressed.into()
            }
            Input::GamepadAxis { axis } => {
                if self.consumed.gamepad_axes.contains(&(gamepad, axis)) {
                    return 0.0.into();
                }

                let value = match gamepad {
                    GamepadDevice::Any => self.gamepads.iter().find_map(|gamepad| {
                        self.gamepad_axis.get_unclamped(GamepadAxis {
                            gamepad,
                            axis_type: axis,
                        })
                    }),
                    GamepadDevice::Id(gamepad) => self.gamepad_axis.get_unclamped(GamepadAxis {
                        gamepad,
                        axis_type: axis,
                    }),
                };

                let value = value.unwrap_or_default();

                if value != 0.0 && consume {
                    self.consumed.gamepad_axes.insert((gamepad, axis));
                }

                value.into()
            }
        }
    }

    fn modifiers_pressed(&self, modifiers: KeyboardModifiers) -> bool {
        if self.consumed.modifiers.intersects(modifiers) {
            return false;
        }

        for (_, modifier) in modifiers.iter_names() {
            let key_codes = match modifier {
                KeyboardModifiers::ALT => [KeyCode::AltLeft, KeyCode::AltRight],
                KeyboardModifiers::CONTROL => [KeyCode::ControlLeft, KeyCode::ControlRight],
                KeyboardModifiers::SHIFT => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
                KeyboardModifiers::SUPER => [KeyCode::SuperLeft, KeyCode::SuperRight],
                _ => unreachable!("iteration should yield only named flags"),
            };

            if !self.key_codes.any_pressed(key_codes) {
                return false;
            }
        }

        true
    }
}

#[derive(Resource, Default)]
struct ConsumedInput {
    ui_wants_keyboard: bool,
    ui_wants_mouse: bool,
    key_codes: HashSet<KeyCode>,
    modifiers: KeyboardModifiers,
    mouse_buttons: HashSet<MouseButton>,
    mouse_motion: bool,
    mouse_wheel: bool,
    gamepad_buttons: HashSet<(GamepadDevice, GamepadButtonType)>,
    gamepad_axes: HashSet<(GamepadDevice, GamepadAxisType)>,
}

impl ConsumedInput {
    fn reset(&mut self) {
        self.ui_wants_keyboard = false;
        self.ui_wants_mouse = false;
        self.key_codes.clear();
        self.modifiers = KeyboardModifiers::empty();
        self.mouse_buttons.clear();
        self.mouse_motion = false;
        self.mouse_wheel = false;
        self.gamepad_buttons.clear();
        self.gamepad_axes.clear();
    }
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, Hash, PartialEq, Eq)]
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
