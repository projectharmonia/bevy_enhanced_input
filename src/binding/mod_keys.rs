use core::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// Keyboard modifiers for both left and right keys.
#[derive(Default, Reflect, Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct ModKeys(u8);

bitflags! {
    impl ModKeys: u8 {
        /// Corresponds to [`KeyCode::ControlLeft`] and [`KeyCode::ControlRight`].
        const CONTROL = 0b00000001;
        /// Corresponds to [`KeyCode::ShiftLeft`] and [`KeyCode::ShiftRight`]
        const SHIFT = 0b00000010;
        /// Corresponds to [`KeyCode::AltLeft`] and [`KeyCode::AltRight`].
        const ALT = 0b00000100;
        /// Corresponds to [`KeyCode::SuperLeft`] and [`KeyCode::SuperRight`].
        const SUPER = 0b00001000;
    }
}

impl Display for ModKeys {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (index, (_, mod_key)) in self.iter_names().enumerate() {
            if index != 0 {
                write!(f, " + ")?;
            }
            match mod_key {
                ModKeys::CONTROL => write!(f, "Ctrl")?,
                ModKeys::SHIFT => write!(f, "Shift")?,
                ModKeys::ALT => write!(f, "Alt")?,
                ModKeys::SUPER => write!(f, "Super")?,
                _ => unreachable!("iteration should yield only named flags"),
            }
        }

        Ok(())
    }
}

impl ModKeys {
    /// Returns an instance with currently active modifiers.
    #[must_use]
    pub fn pressed(keys: &ButtonInput<KeyCode>) -> Self {
        let mut mod_keys = Self::empty();
        for [key1, key2] in Self::all().iter_keys() {
            if keys.any_pressed([key1, key2]) {
                mod_keys |= key1.into();
            }
        }

        mod_keys
    }

    /// Returns an iterator over the key codes corresponding to the set modifier bits.
    ///
    /// Each item contains left and right key codes.
    pub fn iter_keys(self) -> impl Iterator<Item = [KeyCode; 2]> {
        self.iter_names().map(|(_, mod_key)| match mod_key {
            ModKeys::CONTROL => [KeyCode::ControlLeft, KeyCode::ControlRight],
            ModKeys::SHIFT => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            ModKeys::ALT => [KeyCode::AltLeft, KeyCode::AltRight],
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
            KeyCode::ControlLeft | KeyCode::ControlRight => ModKeys::CONTROL,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => ModKeys::SHIFT,
            KeyCode::AltLeft | KeyCode::AltRight => ModKeys::ALT,
            KeyCode::SuperLeft | KeyCode::SuperRight => ModKeys::SUPER,
            _ => ModKeys::empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn pressed_mod_keys() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::ControlLeft);
        keys.press(KeyCode::ShiftLeft);
        keys.press(KeyCode::KeyC);

        let mod_keys = ModKeys::pressed(&keys);
        assert_eq!(mod_keys, ModKeys::CONTROL | ModKeys::SHIFT);
    }

    #[test]
    fn mod_keys_display() {
        assert_eq!(ModKeys::CONTROL.to_string(), "Ctrl");
        assert_eq!(ModKeys::all().to_string(), "Ctrl + Shift + Alt + Super");
        assert_eq!(ModKeys::empty().to_string(), "");
    }
}
