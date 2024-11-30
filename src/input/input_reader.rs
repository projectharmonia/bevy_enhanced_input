use std::hash::Hash;

use bevy::{
    ecs::system::SystemParam,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    utils::HashSet,
};
#[cfg(feature = "egui_priority")]
use bevy_egui::EguiContext;

use super::{GamepadDevice, Input, ModKeys};
use crate::action_value::ActionValue;

/// Reads input from multiple sources.
#[derive(SystemParam)]
pub(crate) struct InputReader<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    mouse_motion_events: EventReader<'w, 's, MouseMotion>,
    mouse_wheel_events: EventReader<'w, 's, MouseWheel>,
    gamepads: Query<'w, 's, &'static Gamepad>,
    consumed: Local<'s, ConsumedInput>,
    gamepad_device: Local<'s, GamepadDevice>,
    mouse_wheel: Local<'s, Vec2>,
    mouse_motion: Local<'s, Vec2>,
    #[cfg(feature = "ui_priority")]
    interactions: Query<'w, 's, &'static Interaction>,
    // In egui mutable reference is required to get contexts,
    // unless `immutable_ctx` feature is enabled.
    #[cfg(feature = "egui_priority")]
    egui: Query<'w, 's, &'static mut EguiContext>,
}

impl InputReader<'_, '_> {
    /// Resets all consumed values and reads mouse events.
    pub(crate) fn update_state(&mut self) {
        self.consumed.reset();

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
            .iter_mut()
            .any(|mut ctx| ctx.get_mut().wants_keyboard_input())
        {
            self.consumed.ui_wants_keyboard = true;
        }

        #[cfg(feature = "egui_priority")]
        if self
            .egui
            .iter_mut()
            .any(|mut ctx| ctx.get_mut().wants_pointer_input())
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
    pub(crate) fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        *self.gamepad_device = gamepad.into();
    }

    /// Returns the [`ActionValue`] for the given [`Input`] if exists.
    ///
    /// See also [`Self::consume`] and [`Self::set_gamepad`].
    pub(crate) fn value(&self, input: impl Into<Input>) -> ActionValue {
        match input.into() {
            Input::Keyboard { key, mod_keys } => {
                let pressed = !self.consumed.ui_wants_keyboard
                    && self.keys.pressed(key)
                    && !self.consumed.keys.contains(&key)
                    && self.mod_keys_pressed(mod_keys);

                pressed.into()
            }
            Input::MouseButton { button, mod_keys } => {
                let pressed = !self.consumed.ui_wants_mouse
                    && self.mouse_buttons.pressed(button)
                    && !self.consumed.mouse_buttons.contains(&button)
                    && self.mod_keys_pressed(mod_keys);

                pressed.into()
            }
            Input::MouseMotion { mod_keys } => {
                if self.consumed.ui_wants_mouse || !self.mod_keys_pressed(mod_keys) {
                    return Vec2::ZERO.into();
                }

                let value = *self.mouse_motion;
                value.into()
            }
            Input::MouseWheel { mod_keys } => {
                if self.consumed.ui_wants_mouse || !self.mod_keys_pressed(mod_keys) {
                    return Vec2::ZERO.into();
                }

                let value = *self.mouse_wheel;
                value.into()
            }
            Input::GamepadButton(button) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: button,
                };

                if self.consumed.gamepad_buttons.contains(&input) {
                    return false.into();
                }

                let pressed = match *self.gamepad_device {
                    GamepadDevice::Any => {
                        self.gamepads.iter().any(|gamepad| gamepad.pressed(button))
                    }
                    GamepadDevice::Single(entity) => self
                        .gamepads
                        .get(entity)
                        .is_ok_and(|gamepad| gamepad.pressed(button)),
                };

                pressed.into()
            }
            Input::GamepadAxis(axis) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: axis,
                };

                if self.consumed.gamepad_axes.contains(&input) {
                    return 0.0.into();
                }

                let value = match *self.gamepad_device {
                    GamepadDevice::Any => self.gamepads.iter().find_map(|gamepad| {
                        gamepad.get_unclamped(axis).filter(|&value| value != 0.0)
                    }),
                    GamepadDevice::Single(entity) => self
                        .gamepads
                        .get(entity)
                        .ok()
                        .and_then(|gamepad| gamepad.get(axis)),
                };

                let value = value.unwrap_or_default();
                value.into()
            }
        }
    }

    fn mod_keys_pressed(&self, mod_keys: ModKeys) -> bool {
        if !mod_keys.is_empty() && self.consumed.ui_wants_keyboard {
            return false;
        }

        if self.consumed.mod_keys.intersects(mod_keys) {
            return false;
        }

        for keys in mod_keys.iter_keys() {
            if !self.keys.any_pressed(keys) {
                return false;
            }
        }

        true
    }

    /// Consumes the input, making it unavailable for [`Self::value`].
    ///
    /// Resets with [`Self::update_state`].
    pub(crate) fn consume(&mut self, input: impl Into<Input>) {
        match input.into() {
            Input::Keyboard { key, mod_keys } => {
                self.consumed.keys.insert(key);
                self.consumed.mod_keys.insert(mod_keys);
            }
            Input::MouseButton { button, mod_keys } => {
                self.consumed.mouse_buttons.insert(button);
                self.consumed.mod_keys.insert(mod_keys);
            }
            Input::MouseMotion { mod_keys } => {
                *self.mouse_motion = Vec2::ZERO;
                self.consumed.mod_keys.insert(mod_keys);
            }
            Input::MouseWheel { mod_keys } => {
                *self.mouse_wheel = Vec2::ZERO;
                self.consumed.mod_keys.insert(mod_keys);
            }
            Input::GamepadButton(button) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: button,
                };

                self.consumed.gamepad_buttons.insert(input);
            }
            Input::GamepadAxis(axis) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: axis,
                };

                self.consumed.gamepad_axes.insert(input);
            }
        }
    }
}

/// Tracks all consumed input from Bevy resources.
///
/// Mouse motion and wheel can be consumed directly since we accumulate them.
#[derive(Resource, Default)]
struct ConsumedInput {
    ui_wants_keyboard: bool,
    ui_wants_mouse: bool,
    keys: HashSet<KeyCode>,
    mod_keys: ModKeys,
    mouse_buttons: HashSet<MouseButton>,
    gamepad_buttons: HashSet<GamepadInput<GamepadButton>>,
    gamepad_axes: HashSet<GamepadInput<GamepadAxis>>,
}

impl ConsumedInput {
    fn reset(&mut self) {
        self.ui_wants_keyboard = false;
        self.ui_wants_mouse = false;
        self.keys.clear();
        self.mod_keys = ModKeys::empty();
        self.mouse_buttons.clear();
        self.gamepad_buttons.clear();
        self.gamepad_axes.clear();
    }
}

/// Input and associated device.
#[derive(Hash, PartialEq, Eq)]
struct GamepadInput<T: Hash + Eq> {
    gamepad: GamepadDevice,
    input: T,
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::system::SystemState, input::mouse::MouseScrollUnit};

    use super::*;
    use crate::{input::InputModKeys, Input};

    #[test]
    fn keyboard() {
        let (mut world, mut state) = init_world();

        let key = KeyCode::Space;
        world.resource_mut::<ButtonInput<KeyCode>>().press(key);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(key), ActionValue::Bool(true));
        assert_eq!(reader.value(KeyCode::Escape), ActionValue::Bool(false));
        assert_eq!(
            reader.value(key.with_mod_keys(ModKeys::ALT)),
            ActionValue::Bool(false)
        );

        reader.consume(key);
        assert_eq!(reader.value(key), ActionValue::Bool(false));
    }

    #[test]
    fn mouse_button() {
        let (mut world, mut state) = init_world();

        let button = MouseButton::Left;
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(button), ActionValue::Bool(true));
        assert_eq!(reader.value(MouseButton::Right), ActionValue::Bool(false));
        assert_eq!(
            reader.value(button.with_mod_keys(ModKeys::CONTROL)),
            ActionValue::Bool(false)
        );

        reader.consume(button);
        assert_eq!(reader.value(button), ActionValue::Bool(false));
    }

    #[test]
    fn mouse_motion() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        world.send_event(MouseMotion { delta: value });

        let input = Input::mouse_motion();
        let mut reader = state.get_mut(&mut world);
        reader.update_state();
        assert_eq!(reader.value(input), ActionValue::Axis2D(value));
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SHIFT)),
            ActionValue::Axis2D(Vec2::ZERO)
        );

        reader.consume(input);
        assert_eq!(reader.value(input), ActionValue::Axis2D(Vec2::ZERO));
    }

    #[test]
    fn mouse_wheel() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        world.send_event(MouseWheel {
            x: value.x,
            y: value.y,
            unit: MouseScrollUnit::Line,
            window: Entity::PLACEHOLDER,
        });

        let input = Input::mouse_wheel();
        let mut reader = state.get_mut(&mut world);
        reader.update_state();
        assert_eq!(reader.value(input), ActionValue::Axis2D(value));
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SUPER)),
            ActionValue::Axis2D(Vec2::ZERO)
        );

        reader.consume(input);
        assert_eq!(reader.value(input), ActionValue::Axis2D(Vec2::ZERO));
    }

    #[test]
    fn gamepad_button() {
        let (mut world, mut state) = init_world();

        let button1 = GamepadButton::South;
        let mut gamepad1 = Gamepad::default();
        gamepad1.digital_mut().press(button1);
        let gamepad_entity = world.spawn(gamepad1).id();

        let button2 = GamepadButton::East;
        let mut gamepad2 = Gamepad::default();
        gamepad2.digital_mut().press(button2);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        reader.set_gamepad(gamepad_entity);
        assert_eq!(reader.value(button1), ActionValue::Bool(true));
        assert_eq!(
            reader.value(button2),
            ActionValue::Bool(false),
            "should read only from `{gamepad_entity:?}`"
        );
        assert_eq!(reader.value(GamepadButton::North), ActionValue::Bool(false));

        reader.consume(button1);
        assert_eq!(reader.value(button1), ActionValue::Bool(false));
    }

    #[test]
    fn any_gamepad_button() {
        let (mut world, mut state) = init_world();

        let button1 = GamepadButton::South;
        let mut gamepad1 = Gamepad::default();
        gamepad1.digital_mut().press(button1);
        world.spawn(gamepad1);

        let button2 = GamepadButton::East;
        let mut gamepad2 = Gamepad::default();
        gamepad2.digital_mut().press(button2);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(button1), ActionValue::Bool(true));
        assert_eq!(reader.value(button2), ActionValue::Bool(true));
        assert_eq!(reader.value(GamepadButton::North), ActionValue::Bool(false));

        reader.consume(button1);
        assert_eq!(reader.value(button1), ActionValue::Bool(false));

        reader.consume(button2);
        assert_eq!(reader.value(button2), ActionValue::Bool(false));
    }

    #[test]
    fn gamepad_axis() {
        let (mut world, mut state) = init_world();

        let value = 1.0;

        let axis1 = GamepadAxis::LeftStickX;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(axis1, value);
        let gamepad_entity = world.spawn(gamepad1).id();

        let axis2 = GamepadAxis::LeftStickY;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(axis2, value);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        reader.set_gamepad(gamepad_entity);
        assert_eq!(reader.value(axis1), ActionValue::Axis1D(1.0));
        assert_eq!(
            reader.value(axis2),
            ActionValue::Axis1D(0.0),
            "should read only from `{gamepad_entity:?}`"
        );
        assert_eq!(
            reader.value(GamepadAxis::RightStickX),
            ActionValue::Axis1D(0.0)
        );

        reader.consume(axis1);
        assert_eq!(reader.value(axis1), ActionValue::Axis1D(0.0));
    }

    #[test]
    fn any_gamepad_axis() {
        let (mut world, mut state) = init_world();

        let value = 1.0;

        let axis1 = GamepadAxis::LeftStickX;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(axis1, value);
        world.spawn(gamepad1);

        let axis2 = GamepadAxis::LeftStickY;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(axis2, value);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(axis1), ActionValue::Axis1D(1.0));
        assert_eq!(reader.value(axis2), ActionValue::Axis1D(1.0));
        assert_eq!(
            reader.value(GamepadAxis::RightStickX),
            ActionValue::Axis1D(0.0)
        );

        reader.consume(axis1);
        assert_eq!(reader.value(axis1), ActionValue::Axis1D(0.0));

        reader.consume(axis2);
        assert_eq!(reader.value(axis2), ActionValue::Axis1D(0.0));
    }

    #[test]
    fn keyboard_with_modifier() {
        let (mut world, mut state) = init_world();

        let key = KeyCode::Space;
        let modifier = KeyCode::ControlLeft;
        let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
        keys.press(modifier);
        keys.press(key);

        let input = key.with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(input), ActionValue::Bool(true));
        assert_eq!(reader.value(key), ActionValue::Bool(true));
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::ALT)),
            ActionValue::Bool(false)
        );
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::CONTROL | ModKeys::ALT)),
            ActionValue::Bool(false)
        );

        reader.consume(input);
        assert_eq!(reader.value(input), ActionValue::Bool(false));

        // Try another key, but with the same modifier that was consumed.
        let other_key = KeyCode::Enter;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(other_key);
        let other_input = other_key.with_mod_keys(modifier.into());
        let reader = state.get_mut(&mut world);
        assert_eq!(reader.value(other_input), ActionValue::Bool(false));
        assert_eq!(reader.value(other_key), ActionValue::Bool(true));
    }

    #[test]
    fn mouse_button_with_modifier() {
        let (mut world, mut state) = init_world();

        let button = MouseButton::Left;
        let modifier = KeyCode::AltLeft;
        world.resource_mut::<ButtonInput<KeyCode>>().press(modifier);
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);

        let input = button.with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(input), ActionValue::Bool(true));
        assert_eq!(reader.value(button), ActionValue::Bool(true));
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::CONTROL)),
            ActionValue::Bool(false)
        );
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::CONTROL | ModKeys::ALT)),
            ActionValue::Bool(false)
        );

        reader.consume(input);
        assert_eq!(reader.value(input), ActionValue::Bool(false));
    }

    #[test]
    fn mouse_motion_with_modifier() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        let modifier = KeyCode::ShiftLeft;
        world.resource_mut::<ButtonInput<KeyCode>>().press(modifier);
        world.send_event(MouseMotion { delta: value });

        let input = Input::mouse_motion().with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        reader.update_state();
        assert_eq!(reader.value(input), ActionValue::Axis2D(value));
        assert_eq!(
            reader.value(input.without_mod_keys()),
            ActionValue::Axis2D(value)
        );
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SUPER)),
            ActionValue::Axis2D(Vec2::ZERO)
        );
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SHIFT | ModKeys::SUPER)),
            ActionValue::Axis2D(Vec2::ZERO)
        );

        reader.consume(input);
        assert_eq!(reader.value(input), ActionValue::Axis2D(Vec2::ZERO));
    }

    #[test]
    fn mouse_wheel_with_modifier() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        let modifier = KeyCode::SuperLeft;
        world.resource_mut::<ButtonInput<KeyCode>>().press(modifier);
        world.send_event(MouseWheel {
            x: value.x,
            y: value.y,
            unit: MouseScrollUnit::Line,
            window: Entity::PLACEHOLDER,
        });

        let input = Input::mouse_wheel().with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        reader.update_state();
        assert_eq!(reader.value(input), ActionValue::Axis2D(value));
        assert_eq!(
            reader.value(input.without_mod_keys()),
            ActionValue::Axis2D(value)
        );
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SHIFT)),
            ActionValue::Axis2D(Vec2::ZERO)
        );
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SHIFT | ModKeys::SUPER)),
            ActionValue::Axis2D(Vec2::ZERO)
        );

        reader.consume(input);
        assert_eq!(reader.value(input), ActionValue::Axis2D(Vec2::ZERO));
    }

    #[test]
    fn ui_input() {
        let (mut world, mut state) = init_world();

        let key = KeyCode::Space;
        let button = MouseButton::Left;
        world.resource_mut::<ButtonInput<KeyCode>>().press(key);
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);
        world.send_event(MouseMotion { delta: Vec2::ONE });
        world.send_event(MouseWheel {
            x: 1.0,
            y: 1.0,
            unit: MouseScrollUnit::Line,
            window: Entity::PLACEHOLDER,
        });

        let mut reader = state.get_mut(&mut world);
        reader.update_state();
        reader.consumed.ui_wants_keyboard = true;
        reader.consumed.ui_wants_mouse = true;

        assert_eq!(reader.value(key), ActionValue::Bool(false));
        assert_eq!(reader.value(button), ActionValue::Bool(false));
        assert_eq!(
            reader.value(Input::mouse_motion()),
            ActionValue::Axis2D(Vec2::ZERO)
        );
        assert_eq!(
            reader.value(Input::mouse_wheel()),
            ActionValue::Axis2D(Vec2::ZERO)
        );
    }

    fn init_world<'w, 's>() -> (World, SystemState<InputReader<'w, 's>>) {
        let mut world = World::new();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<Events<MouseMotion>>();
        world.init_resource::<Events<MouseWheel>>();
        world.init_resource::<ButtonInput<GamepadButton>>();
        world.init_resource::<Axis<GamepadAxis>>();

        let state = SystemState::<InputReader>::new(&mut world);

        (world, state)
    }
}
