use std::hash::Hash;

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
    params: Local<'s, ReaderParams>,
    mouse_wheel: Local<'s, Vec2>,
    mouse_motion: Local<'s, Vec2>,
    #[cfg(feature = "ui_priority")]
    interactions: Query<'w, 's, &'static Interaction>,
    #[cfg(feature = "egui_priority")]
    egui: Query<'w, 's, &'static EguiContext>,
}

impl InputReader<'_, '_> {
    /// Resets all consumed values and reads mouse events.
    pub(super) fn update_state(&mut self) {
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

        // Mouse motion and wheel input need to be accumulated
        // because they only exist as events, and subsequent reads
        // will return zero.
        *self.mouse_motion = self
            .mouse_motion_events
            .read()
            .map(|event| event.delta)
            .sum();
        *self.mouse_wheel = self
            .mouse_wheel_events
            .read()
            .map(|event| Vec2::new(event.x, event.y))
            .sum();
    }

    /// Assignes a gamepad from which [`Self::value`] should read input.
    pub(super) fn set_gamepad(&mut self, gamepad: GamepadDevice) {
        self.params.gamepad = gamepad;
    }

    /// Enables or disables consuming in [`Self::value`].
    ///
    /// If the value is consumed, will unavailable for subsequent calls.
    pub(super) fn set_consume_input(&mut self, consume_input: bool) {
        self.params.consume_input = consume_input;
    }

    /// Returns the [`ActionValue`] for the given [`Input`] if exists.
    ///
    /// See also [`Self::set_consume_input`] and [`Self::set_gamepad`].
    pub(super) fn value(&mut self, input: impl Into<Input>) -> ActionValue {
        match input.into() {
            Input::Keyboard {
                key_code,
                modifiers,
            } => {
                let pressed = !self.consumed.ui_wants_keyboard
                    && self.key_codes.pressed(key_code)
                    && !self.consumed.key_codes.contains(&key_code)
                    && self.modifiers_pressed(modifiers);

                if pressed && self.params.consume_input {
                    self.consumed.key_codes.insert(key_code);
                    self.consumed.modifiers.insert(modifiers);
                }

                pressed.into()
            }
            Input::MouseButton { button, modifiers } => {
                let pressed = !self.consumed.ui_wants_mouse
                    && self.mouse_buttons.pressed(button)
                    && !self.consumed.mouse_buttons.contains(&button)
                    && self.modifiers_pressed(modifiers);

                if pressed && self.params.consume_input {
                    self.consumed.mouse_buttons.insert(button);
                    self.consumed.modifiers.insert(modifiers);
                }

                pressed.into()
            }
            Input::MouseMotion { modifiers } => {
                if self.consumed.ui_wants_mouse || !self.modifiers_pressed(modifiers) {
                    return Vec2::ZERO.into();
                }

                let value = *self.mouse_motion;
                if self.params.consume_input {
                    *self.mouse_motion = Vec2::ZERO;
                }

                value.into()
            }
            Input::MouseWheel { modifiers } => {
                if self.consumed.ui_wants_mouse || !self.modifiers_pressed(modifiers) {
                    return Vec2::ZERO.into();
                }

                let value = *self.mouse_wheel;
                if self.params.consume_input {
                    *self.mouse_wheel = Vec2::ZERO;
                }

                value.into()
            }
            Input::GamepadButton { button } => {
                let input = GamepadInput {
                    gamepad: self.params.gamepad,
                    input: button,
                };

                if self.consumed.gamepad_buttons.contains(&input) {
                    return false.into();
                }

                let pressed = match self.params.gamepad {
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

                if pressed && self.params.consume_input {
                    self.consumed.gamepad_buttons.insert(input);
                }

                pressed.into()
            }
            Input::GamepadAxis { axis } => {
                let input = GamepadInput {
                    gamepad: self.params.gamepad,
                    input: axis,
                };

                if self.consumed.gamepad_axes.contains(&input) {
                    return 0.0.into();
                }

                let value = match self.params.gamepad {
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

                if value != 0.0 && self.params.consume_input {
                    self.consumed.gamepad_axes.insert(input);
                }

                value.into()
            }
        }
    }

    fn modifiers_pressed(&self, modifiers: Modifiers) -> bool {
        if !modifiers.is_empty() && self.consumed.ui_wants_keyboard {
            return false;
        }

        if self.consumed.modifiers.intersects(modifiers) {
            return false;
        }

        for key_codes in modifiers.iter_key_codes() {
            if !self.key_codes.any_pressed(key_codes) {
                return false;
            }
        }

        true
    }
}

/// Tracks all consumed input from Bevy resources.
///
/// Mouse motion and wheel can be consumed directly since we accumulate them.
#[derive(Resource, Default)]
struct ConsumedInput {
    ui_wants_keyboard: bool,
    ui_wants_mouse: bool,
    key_codes: HashSet<KeyCode>,
    modifiers: Modifiers,
    mouse_buttons: HashSet<MouseButton>,
    gamepad_buttons: HashSet<GamepadInput<GamepadButtonType>>,
    gamepad_axes: HashSet<GamepadInput<GamepadAxisType>>,
}

impl ConsumedInput {
    fn reset(&mut self) {
        self.ui_wants_keyboard = false;
        self.ui_wants_mouse = false;
        self.key_codes.clear();
        self.modifiers = Modifiers::empty();
        self.mouse_buttons.clear();
        self.gamepad_buttons.clear();
        self.gamepad_axes.clear();
    }
}

/// Similar to [`GamepadButton`] or [`GamepadAxis`],
/// but uses [`GamepadDevice`] that can map any gamepad.
#[derive(Hash, PartialEq, Eq)]
struct GamepadInput<T: Hash + Eq> {
    gamepad: GamepadDevice,
    input: T,
}

/// Parameters for [`InputReader`].
#[derive(Default)]
struct ReaderParams {
    /// Whether to consume input after reading from [`InputReader::value`].
    consume_input: bool,

    /// Associated gamepad.
    gamepad: GamepadDevice,
}

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
    pub fn iter_key_codes(self) -> impl Iterator<Item = [KeyCode; 2]> {
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
    /// Keyboard button, will be captured as [`ActionValue::Bool`].
    Keyboard {
        key_code: KeyCode,
        modifiers: Modifiers,
    },
    /// Mouse button, will be captured as [`ActionValue::Bool`].
    MouseButton {
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// Mouse movement, will be captured as [`ActionValue::Axis2D`].
    MouseMotion { modifiers: Modifiers },
    /// Mouse wheel, will be captured as [`ActionValue::Axis1D`].
    MouseWheel { modifiers: Modifiers },
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

#[cfg(test)]
mod tests {
    use bevy::ecs::system::SystemState;

    use super::*;

    #[test]
    fn key_code() {
        let (mut world, mut state) = init_world();

        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);

        let mut reader = state.get(&world);
        assert_eq!(reader.value(KeyCode::Space), ActionValue::Bool(true));
        assert_eq!(reader.value(KeyCode::Space), ActionValue::Bool(true));

        reader.set_consume_input(true);
        assert_eq!(reader.value(KeyCode::Space), ActionValue::Bool(true));
        assert_eq!(reader.value(KeyCode::Space), ActionValue::Bool(false));
    }

    fn init_world<'w, 's>() -> (World, SystemState<InputReader<'w, 's>>) {
        let mut world = World::new();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<Events<MouseMotion>>();
        world.init_resource::<Events<MouseWheel>>();
        world.init_resource::<ButtonInput<GamepadButton>>();
        world.init_resource::<Axis<GamepadAxis>>();
        world.init_resource::<Gamepads>();

        let state = SystemState::<InputReader>::new(&mut world);

        (world, state)
    }
}
