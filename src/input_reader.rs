use alloc::vec::Vec;
use core::hash::Hash;

use bevy::{
    ecs::system::SystemParam,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    utils::HashSet,
};

use crate::{
    action_value::ActionValue,
    input::{GamepadDevice, Input, ModKeys},
};

/// Input state for actions.
///
/// Actions can read input values and optionally consume them without affecting Bevy input resources.
#[derive(SystemParam)]
pub(crate) struct InputReader<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    mouse_motion: Res<'w, AccumulatedMouseMotion>,
    mouse_scroll: Res<'w, AccumulatedMouseScroll>,
    gamepads: Query<'w, 's, &'static Gamepad>,
    action_sources: Res<'w, ActionSources>,
    consumed: Local<'s, ConsumedInput>,
    reset_input: ResMut<'w, ResetInput>,
    gamepad_device: Local<'s, GamepadDevice>,
}

impl InputReader<'_, '_> {
    /// Resets all consumed values and reads mouse events.
    pub(crate) fn update_state(&mut self) {
        self.consumed.reset();

        // Temporary take the original value to avoid issues with the borrow checker.
        let mut reset_input = core::mem::take(&mut *self.reset_input);
        reset_input.retain(|&input| {
            if self.value(input).as_bool() {
                self.consume(input);
                true
            } else {
                false
            }
        });
        *self.reset_input = reset_input;
    }

    /// Assigns a gamepad from which [`Self::value`] should read input.
    pub(crate) fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        *self.gamepad_device = gamepad.into();
    }

    /// Returns the [`ActionValue`] for the given [`Input`].
    ///
    /// See also [`Self::consume`] and [`Self::set_gamepad`].
    pub(crate) fn value(&self, input: impl Into<Input>) -> ActionValue {
        match input.into() {
            Input::Keyboard { key, mod_keys } => {
                let pressed = self.action_sources.keyboard
                    && self.keys.pressed(key)
                    && !self.consumed.keys.contains(&key)
                    && self.mod_keys_pressed(mod_keys);

                pressed.into()
            }
            Input::MouseButton { button, mod_keys } => {
                let pressed = self.action_sources.mouse_buttons
                    && self.mouse_buttons.pressed(button)
                    && !self.consumed.mouse_buttons.contains(&button)
                    && self.mod_keys_pressed(mod_keys);

                pressed.into()
            }
            Input::MouseMotion { mod_keys } => {
                if !self.action_sources.mouse_motion
                    || !self.mod_keys_pressed(mod_keys)
                    || self.consumed.mouse_motion
                {
                    return Vec2::ZERO.into();
                }

                self.mouse_motion.delta.into()
            }
            Input::MouseWheel { mod_keys } => {
                if !self.action_sources.mouse_wheel
                    || !self.mod_keys_pressed(mod_keys)
                    || self.consumed.mouse_wheel
                {
                    return Vec2::ZERO.into();
                }

                self.mouse_scroll.delta.into()
            }
            Input::GamepadButton(button) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: button,
                };

                if !self.action_sources.gamepad_button
                    || self.consumed.gamepad_buttons.contains(&input)
                {
                    return 0.0.into();
                }

                let value = match *self.gamepad_device {
                    GamepadDevice::Any => self
                        .gamepads
                        .iter()
                        .filter_map(|gamepad| gamepad.get(button))
                        .find(|&value| value != 0.0),
                    GamepadDevice::Single(entity) => self
                        .gamepads
                        .get(entity)
                        .ok()
                        .and_then(|gamepad| gamepad.get(button)),
                };

                value.unwrap_or_default().into()
            }
            Input::GamepadAxis(axis) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: axis,
                };

                if !self.action_sources.gamepad_axis || self.consumed.gamepad_axes.contains(&input)
                {
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
        if !mod_keys.is_empty() && !self.action_sources.keyboard {
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
                self.consumed.mouse_motion = true;
                self.consumed.mod_keys.insert(mod_keys);
            }
            Input::MouseWheel { mod_keys } => {
                self.consumed.mouse_wheel = true;
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

/// Configures which input sources are visible to actions.
///
/// Defaults to `true` for all values.
///
/// Could be used to prevent actions from being triggered
/// while interacting with the UI.
///
/// # Examples
///
/// Disables mouse buttons for actions when the cursor hovers a node with
/// an `Interaction` component. It's a required component for `Button`,
/// but you can add it to any UI node to disable specific actions on hover.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// # let mut app = App::new();
/// app.add_systems(PreUpdate, disable_mouse.before(EnhancedInputSystem));
///
/// fn disable_mouse(
///     mut action_sources: ResMut<ActionSources>,
///     interactions: Query<&Interaction>,
/// ) {
///     let mouse_used = interactions.iter().all(|&interaction| interaction == Interaction::None);
///     action_sources.mouse_buttons = !mouse_used;
///     action_sources.mouse_wheel = !mouse_used;
/// }
/// ```
#[derive(Resource, Reflect)]
pub struct ActionSources {
    pub keyboard: bool,
    pub mouse_buttons: bool,
    pub mouse_motion: bool,
    pub mouse_wheel: bool,
    pub gamepad_button: bool,
    pub gamepad_axis: bool,
}

impl Default for ActionSources {
    fn default() -> Self {
        Self {
            keyboard: true,
            mouse_buttons: true,
            mouse_motion: true,
            mouse_wheel: true,
            gamepad_button: true,
            gamepad_axis: true,
        }
    }
}

/// Tracks all consumed input from Bevy resources.
#[derive(Resource, Default)]
struct ConsumedInput {
    keys: HashSet<KeyCode>,
    mod_keys: ModKeys,
    mouse_buttons: HashSet<MouseButton>,
    mouse_motion: bool,
    mouse_wheel: bool,
    gamepad_buttons: HashSet<GamepadInput<GamepadButton>>,
    gamepad_axes: HashSet<GamepadInput<GamepadAxis>>,
}

impl ConsumedInput {
    fn reset(&mut self) {
        self.keys.clear();
        self.mod_keys = ModKeys::empty();
        self.mouse_buttons.clear();
        self.mouse_motion = false;
        self.mouse_wheel = false;
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

/// Stores inputs that will be ignored until they return zero.
///
/// Once the input becomes zero, it will be automatically removed and no longer ignored.
#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct ResetInput(Vec<Input>);

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::system::SystemState,
        input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    };

    use super::*;
    use crate::{Input, input::InputModKeys};

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
        world.insert_resource(AccumulatedMouseMotion { delta: value });

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
        world.insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: value,
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

        let value = 1.0;
        let button1 = GamepadButton::South;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(button1, value);
        let gamepad_entity = world.spawn(gamepad1).id();

        let button2 = GamepadButton::East;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(button2, value);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        reader.set_gamepad(gamepad_entity);
        assert_eq!(reader.value(button1), ActionValue::Axis1D(value));
        assert_eq!(
            reader.value(button2),
            ActionValue::Axis1D(0.0),
            "should read only from `{gamepad_entity:?}`"
        );
        assert_eq!(reader.value(GamepadButton::North), ActionValue::Axis1D(0.0));

        reader.consume(button1);
        assert_eq!(reader.value(button1), ActionValue::Axis1D(0.0));
    }

    #[test]
    fn any_gamepad_button() {
        let (mut world, mut state) = init_world();

        let value = 1.0;
        let button1 = GamepadButton::South;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(button1, value);
        world.spawn(gamepad1);

        let button2 = GamepadButton::East;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(button2, value);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(button1), ActionValue::Axis1D(value));
        assert_eq!(reader.value(button2), ActionValue::Axis1D(value));
        assert_eq!(reader.value(GamepadButton::North), ActionValue::Axis1D(0.0));

        reader.consume(button1);
        assert_eq!(reader.value(button1), ActionValue::Axis1D(0.0));

        reader.consume(button2);
        assert_eq!(reader.value(button2), ActionValue::Axis1D(0.0));
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
        assert_eq!(reader.value(axis1), ActionValue::Axis1D(value));
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
        assert_eq!(reader.value(axis1), ActionValue::Axis1D(value));
        assert_eq!(reader.value(axis2), ActionValue::Axis1D(value));
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
        world.insert_resource(AccumulatedMouseMotion { delta: value });

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
        world.insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: value,
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
    fn sources() {
        let (mut world, mut state) = init_world();

        let key = KeyCode::Space;
        let mouse_button = MouseButton::Left;
        let gamepad_button = GamepadButton::South;
        let axis = GamepadAxis::LeftStickX;

        world.resource_mut::<ButtonInput<KeyCode>>().press(key);
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(mouse_button);

        world.insert_resource(AccumulatedMouseMotion { delta: Vec2::ONE });
        world.insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: Vec2::ONE,
        });

        let mut gamepad = Gamepad::default();
        gamepad.analog_mut().set(axis, 1.0);
        gamepad.analog_mut().set(gamepad_button, 1.0);
        world.spawn(gamepad);

        let mut action_sources = world.resource_mut::<ActionSources>();
        action_sources.keyboard = false;
        action_sources.mouse_buttons = false;
        action_sources.mouse_motion = false;
        action_sources.mouse_wheel = false;
        action_sources.gamepad_button = false;
        action_sources.gamepad_axis = false;

        let mut reader = state.get_mut(&mut world);
        reader.update_state();

        assert_eq!(reader.value(key), ActionValue::Bool(false));
        assert_eq!(reader.value(mouse_button), ActionValue::Bool(false));
        assert_eq!(
            reader.value(Input::mouse_motion()),
            ActionValue::Axis2D(Vec2::ZERO)
        );
        assert_eq!(
            reader.value(Input::mouse_wheel()),
            ActionValue::Axis2D(Vec2::ZERO)
        );
        assert_eq!(reader.value(gamepad_button), ActionValue::Axis1D(0.0));
        assert_eq!(reader.value(axis), ActionValue::Axis1D(0.0));
    }

    fn init_world<'w, 's>() -> (World, SystemState<InputReader<'w, 's>>) {
        let mut world = World::new();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<Events<MouseMotion>>();
        world.init_resource::<Events<MouseWheel>>();
        world.init_resource::<ButtonInput<GamepadButton>>();
        world.init_resource::<Axis<GamepadAxis>>();
        world.init_resource::<AccumulatedMouseMotion>();
        world.init_resource::<AccumulatedMouseScroll>();
        world.init_resource::<ResetInput>();
        world.init_resource::<ActionSources>();

        let state = SystemState::<InputReader>::new(&mut world);

        (world, state)
    }
}
