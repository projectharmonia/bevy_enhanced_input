use std::hash::Hash;

use bevy::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// Inputs that can be associated with an
/// [`InputAction`](super::input_context::input_action::InputAction).
///
/// [Input modifiers](super::input_context::input_modifier) can change the captured dimension.
///
/// If the action's dimension differs from the captured input, it will be converted using
/// [`ActionValue::convert`](crate::action_value::ActionValue::convert).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Reflect, PartialEq)]
pub enum Input {
    /// Keyboard button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue::Bool).
    Keyboard { key: KeyCode, mod_keys: ModKeys },
    /// Mouse button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue::Bool).
    MouseButton {
        button: MouseButton,
        mod_keys: ModKeys,
    },
    /// Mouse movement, will be captured as
    /// [`ActionValue::Axis2D`](crate::action_value::ActionValue::Axis2D).
    MouseMotion { mod_keys: ModKeys },
    /// Mouse wheel, will be captured as
    /// [`ActionValue::Axis1D`](crate::action_value::ActionValue::Axis1D).
    MouseWheel { mod_keys: ModKeys },
    /// Gamepad button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue::Bool).
    GamepadButton(GamepadButton),
    /// Gamepad stick axis, will be captured as
    /// [`ActionValue::Axis1D`](crate::action_value::ActionValue::Axis1D).
    GamepadAxis(GamepadAxis),
}

impl Input {
    /// Returns [`Input::MouseMotion`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_motion() -> Self {
        Self::MouseMotion {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns [`Input::MouseWheel`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_wheel() -> Self {
        Self::MouseWheel {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns new instance without any keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Self::GamepadButton`] or [`Self::GamepadAxis`].
    #[must_use]
    pub fn without_mod_keys(self) -> Self {
        self.with_mod_keys(ModKeys::empty())
    }
}

impl From<KeyCode> for Input {
    fn from(key: KeyCode) -> Self {
        Self::Keyboard {
            key,
            mod_keys: Default::default(),
        }
    }
}

impl From<MouseButton> for Input {
    fn from(button: MouseButton) -> Self {
        Self::MouseButton {
            button,
            mod_keys: Default::default(),
        }
    }
}

impl From<GamepadButton> for Input {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

impl From<GamepadAxis> for Input {
    fn from(value: GamepadAxis) -> Self {
        Self::GamepadAxis(value)
    }
}

/// A trait to ergonomically assign keyboard modifiers to any type that can be converted into an input.
pub trait InputModKeys {
    /// Returns an input with assigned keyboard modifiers.
    #[must_use]
    fn with_mod_keys(self, mod_keys: ModKeys) -> Input;
}

impl<I: Into<Input>> InputModKeys for I {
    /// Returns new instance with the replaced keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Input::GamepadButton`] or [`Input::GamepadAxis`].
    fn with_mod_keys(self, mod_keys: ModKeys) -> Input {
        match self.into() {
            Input::Keyboard { key, .. } => Input::Keyboard { key, mod_keys },
            Input::MouseButton { button, .. } => Input::MouseButton { button, mod_keys },
            Input::MouseMotion { .. } => Input::MouseMotion { mod_keys },
            Input::MouseWheel { .. } => Input::MouseWheel { mod_keys },
            Input::GamepadButton { .. } | Input::GamepadAxis { .. } => {
                panic!("keyboard modifiers can't be applied to gamepads")
            }
        }
    }
}

/// Keyboard modifiers for both left and right keys.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct ModKeys(u8);

bitflags! {
    impl ModKeys: u8 {
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

impl ModKeys {
    /// Returns an iterator over the key codes corresponding to the set modifier bits.
    ///
    /// Each item contains left and right key codes.
    pub fn iter_keys(self) -> impl Iterator<Item = [KeyCode; 2]> {
        self.iter_names().map(|(_, mod_key)| match mod_key {
            ModKeys::ALT => [KeyCode::AltLeft, KeyCode::AltRight],
            ModKeys::CONTROL => [KeyCode::ControlLeft, KeyCode::ControlRight],
            ModKeys::SHIFT => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            ModKeys::SUPER => [KeyCode::SuperLeft, KeyCode::SuperRight],
            _ => unreachable!("iteration should yield only named flags"),
        })
    }
}

impl From<KeyCode> for ModKeys {
    /// Converts key into a named modifier
    ///
    /// Returns [`ModKeys::empty`] if the key is not a modifier.
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::AltLeft | KeyCode::AltRight => ModKeys::ALT,
            KeyCode::ControlLeft | KeyCode::ControlRight => ModKeys::CONTROL,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => ModKeys::SHIFT,
            KeyCode::SuperLeft | KeyCode::SuperRight => ModKeys::SUPER,
            _ => ModKeys::empty(),
        }
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
    Single(Entity),
}

impl GamepadDevice {
    /// Returns `true` if this device matches the specified gamepad entity.
    pub fn matches(self, gamepad_entity: Entity) -> bool {
        match self {
            GamepadDevice::Any => true,
            GamepadDevice::Single(entity) => entity == gamepad_entity,
        }
    }
}

impl From<Entity> for GamepadDevice {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}
