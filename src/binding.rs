pub mod mod_keys;
pub mod relationship;

use core::fmt::{self, Display, Formatter};

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// A an input bound to an [`Action<C>`].
///
/// Should be stored on a separate entity from the action,
/// and related to it using the [`BindingOf`] relationship.
///
/// [Input modifiers](crate::modifier) can change the captured dimension.
///
/// If the action's dimension differs from the captured input, it will be converted using
/// [`ActionValue::convert`](crate::action::value::ActionValue::convert).
#[derive(Component, Reflect, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[component(on_insert = reset_first_activation, immutable)]
#[require(FirstActivation)]
pub enum Binding {
    /// Keyboard button, captured as [`ActionValue::Bool`].
    Keyboard { key: KeyCode, mod_keys: ModKeys },
    /// Mouse button, captured as [`ActionValue::Bool`].
    MouseButton {
        button: MouseButton,
        mod_keys: ModKeys,
    },
    /// Mouse movement, captured as [`ActionValue::Axis2D`].
    MouseMotion { mod_keys: ModKeys },
    /// Mouse wheel, captured as [`ActionValue::Axis2D`].
    ///
    /// In Bevy vertical scroll maps to the Y axis. If you want to bind vertical scroll
    /// to an action with [`ActionValue::Axis1D`], apply [`SwizzleAxis::YXZ`] modifier.
    MouseWheel { mod_keys: ModKeys },
    /// Gamepad button, captured as [`ActionValue::Axis1D`].
    GamepadButton(GamepadButton),
    /// Gamepad stick axis, captured as [`ActionValue::Axis1D`].
    GamepadAxis(GamepadAxis),
    /// Doesn't correspond to any input, captured as [`ActionValue::Bool`] with `false`.
    ///
    /// Useful for expressing empty bindings in [presets](crate::preset).
    None,
}

impl Binding {
    /// Returns [`Self::MouseMotion`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_motion() -> Self {
        Self::MouseMotion {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns [`Self::MouseWheel`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_wheel() -> Self {
        Self::MouseWheel {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns the amount of associated keyboard modifiers.
    #[must_use]
    pub fn mod_keys_count(self) -> usize {
        self.mod_keys().iter_names().count()
    }

    /// Returns associated keyboard modifiers.
    #[must_use]
    pub const fn mod_keys(self) -> ModKeys {
        match self {
            Binding::Keyboard { mod_keys, .. }
            | Binding::MouseButton { mod_keys, .. }
            | Binding::MouseMotion { mod_keys }
            | Binding::MouseWheel { mod_keys } => mod_keys,
            Binding::GamepadButton(_) | Binding::GamepadAxis(_) | Binding::None => ModKeys::empty(),
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

impl Display for Binding {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mod_keys = self.mod_keys();
        if !mod_keys.is_empty() {
            write!(f, "{mod_keys} + ")?;
        }

        match self {
            Binding::Keyboard { key, .. } => write!(f, "{key:?}"),
            Binding::MouseButton { button, .. } => write!(f, "Mouse {button:?}"),
            Binding::MouseMotion { .. } => write!(f, "Mouse Motion"),
            Binding::MouseWheel { .. } => write!(f, "Scroll Wheel"),
            Binding::GamepadButton(gamepad_button) => write!(f, "{gamepad_button:?}"),
            Binding::GamepadAxis(gamepad_axis) => write!(f, "{gamepad_axis:?}"),
            Binding::None => write!(f, "None"),
        }
    }
}

impl From<KeyCode> for Binding {
    fn from(key: KeyCode) -> Self {
        Self::Keyboard {
            key,
            mod_keys: Default::default(),
        }
    }
}

impl From<MouseButton> for Binding {
    fn from(button: MouseButton) -> Self {
        Self::MouseButton {
            button,
            mod_keys: Default::default(),
        }
    }
}

impl From<GamepadButton> for Binding {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

impl From<GamepadAxis> for Binding {
    fn from(value: GamepadAxis) -> Self {
        Self::GamepadAxis(value)
    }
}

/// A trait to ergonomically assign keyboard modifiers to any type that can be converted into a [`Binding`].
pub trait InputModKeys {
    /// Returns a binding with assigned keyboard modifiers.
    #[must_use]
    fn with_mod_keys(self, mod_keys: ModKeys) -> Binding;
}

impl<I: Into<Binding>> InputModKeys for I {
    /// Returns new instance with the replaced keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Binding::GamepadButton`], [`Binding::GamepadAxis`] or [`Binding::None`].
    fn with_mod_keys(self, mod_keys: ModKeys) -> Binding {
        match self.into() {
            Binding::Keyboard { key, .. } => Binding::Keyboard { key, mod_keys },
            Binding::MouseButton { button, .. } => Binding::MouseButton { button, mod_keys },
            Binding::MouseMotion { .. } => Binding::MouseMotion { mod_keys },
            Binding::MouseWheel { .. } => Binding::MouseWheel { mod_keys },
            Binding::GamepadButton { .. } | Binding::GamepadAxis { .. } | Binding::None => {
                panic!("keyboard modifiers can be applied only to mouse and keyboard")
            }
        }
    }
}

fn reset_first_activation(mut world: DeferredWorld, ctx: HookContext) {
    let mut first_activation = world.get_mut::<FirstActivation>(ctx.entity).unwrap();
    **first_activation = true;
}

/// Tracks whether the input defined by [`Binding`] was active at least once.
///
/// Used to prevent newly created contexts from reacting to currently active inputs
/// until they're released.
///
/// Used only if [`ActionSettings::require_reset`] is set.
#[derive(Component, Deref, DerefMut, Default)]
pub(crate) struct FirstActivation(bool);

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn input_display() {
        assert_eq!(
            Binding::Keyboard {
                key: KeyCode::KeyA,
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "KeyA"
        );
        assert_eq!(
            Binding::Keyboard {
                key: KeyCode::KeyA,
                mod_keys: ModKeys::CONTROL
            }
            .to_string(),
            "Ctrl + KeyA"
        );
        assert_eq!(
            Binding::MouseButton {
                button: MouseButton::Left,
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Mouse Left"
        );
        assert_eq!(
            Binding::MouseMotion {
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Mouse Motion"
        );
        assert_eq!(
            Binding::MouseWheel {
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Scroll Wheel"
        );
        assert_eq!(
            Binding::GamepadAxis(GamepadAxis::LeftStickX).to_string(),
            "LeftStickX"
        );
        assert_eq!(
            Binding::GamepadButton(GamepadButton::North).to_string(),
            "North"
        );
    }
}
