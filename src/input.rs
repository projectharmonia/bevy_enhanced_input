pub(super) mod input_reader;

use std::hash::Hash;

use bevy::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Keyboard modifiers for both left and right keys.
    #[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Modifiers: u8 {
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

impl Modifiers {
    /// Returns an iterator over the key codes corresponding to the set modifier bits.
    ///
    /// Each item contains left and right key codes.
    pub fn iter_keys(self) -> impl Iterator<Item = [KeyCode; 2]> {
        self.iter_names().map(|(_, modifier)| match modifier {
            Modifiers::ALT => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifiers::CONTROL => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifiers::SHIFT => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifiers::SUPER => [KeyCode::SuperLeft, KeyCode::SuperRight],
            _ => unreachable!("iteration should yield only named flags"),
        })
    }
}

impl From<KeyCode> for Modifiers {
    /// Converts key into a named modifier
    ///
    /// Returns [`Modifiers::empty`] if the key is not a modifier.
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::AltLeft | KeyCode::AltRight => Modifiers::ALT,
            KeyCode::ControlLeft | KeyCode::ControlRight => Modifiers::CONTROL,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Modifiers::SHIFT,
            KeyCode::SuperLeft | KeyCode::SuperRight => Modifiers::SUPER,
            _ => Modifiers::empty(),
        }
    }
}

/// Inputs that can be associated with an
/// [`InputAction`](super::input_context::input_action::InputAction).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Input {
    /// Keyboard button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue).
    Keyboard { key: KeyCode, modifiers: Modifiers },
    /// Mouse button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue).
    MouseButton {
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// Mouse movement, will be captured as
    /// [`ActionValue::Axis2D`](crate::action_value::ActionValue).
    MouseMotion { modifiers: Modifiers },
    /// Mouse wheel, will be captured as
    /// [`ActionValue::Axis1D`](crate::action_value::ActionValue).
    MouseWheel { modifiers: Modifiers },
    /// Gamepad button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue).
    GamepadButton { button: GamepadButtonType },
    /// Gamepad stick axis, will be captured as
    /// [`ActionValue::Axis1D`](crate::action_value::ActionValue).
    GamepadAxis { axis: GamepadAxisType },
}

impl Input {
    /// Returns [`Input::MouseMotion`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_motion() -> Self {
        Self::MouseMotion {
            modifiers: Modifiers::empty(),
        }
    }

    /// Returns [`Input::MouseWheel`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_wheel() -> Self {
        Self::MouseWheel {
            modifiers: Modifiers::empty(),
        }
    }

    /// Returns new instance without any keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Self::GamepadButton`] or [`Self::GamepadAxis`].
    #[must_use]
    pub const fn without_modifiers(self) -> Self {
        self.with_modifiers(Modifiers::empty())
    }

    /// Returns new instance with the replaced keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Self::GamepadButton`] or [`Self::GamepadAxis`].
    #[must_use]
    pub const fn with_modifiers(self, modifiers: Modifiers) -> Self {
        match self {
            Input::Keyboard { key, .. } => Self::Keyboard { key, modifiers },
            Input::MouseButton { button, .. } => Self::MouseButton { button, modifiers },
            Input::MouseMotion { .. } => Self::MouseMotion { modifiers },
            Input::MouseWheel { .. } => Self::MouseWheel { modifiers },
            Input::GamepadButton { .. } | Input::GamepadAxis { .. } => {
                panic!("keyboard modifiers can't be applied to gamepads")
            }
        }
    }
}

impl From<KeyCode> for Input {
    fn from(key: KeyCode) -> Self {
        Self::Keyboard {
            key,
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
    ///
    /// [`ActionValue`]: crate::action_value::ActionValue
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
