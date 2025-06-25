use alloc::vec::Vec;
use core::{any::TypeId, hash::Hash, iter, mem};

use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemParam},
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    platform::collections::HashSet,
    prelude::*,
    utils::TypeIdMap,
};
use log::trace;

use crate::prelude::*;

pub(crate) fn update_pending(mut reader: InputReader) {
    reader.update_pending();
}

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
    consumed: ResMut<'w, ConsumedInputs>,
    pending: ResMut<'w, PendingInputs>,
    gamepad_device: Local<'s, GamepadDevice>,
    skip_ignore_check: Local<'s, bool>,
}

impl InputReader<'_, '_> {
    /// Updates list of inputs that are waiting for reset.
    pub(crate) fn update_pending(&mut self) {
        // Updated before context-consumed inputs,
        // which may still reference inputs added to the pending.
        *self.skip_ignore_check = true;

        // Temporary take the original value to avoid issues with the borrow checker.
        let mut pending = mem::take(&mut *self.pending);
        pending.ignored.clear();
        pending.inputs.retain(|&input| {
            if self.value(input).as_bool() {
                pending.ignored.add(input, *self.gamepad_device);
                true
            } else {
                trace!("'{input}' reset and no longer ignored");
                false
            }
        });
        *self.pending = pending;

        *self.skip_ignore_check = false
    }

    /// Clears all consumed values from the given schedule.
    pub(crate) fn clear_consumed<S: ScheduleLabel>(&mut self) {
        self.consumed.entry(TypeId::of::<S>()).or_default().clear();
    }

    /// Assigns a gamepad from which [`Self::value`] should read input.
    pub(crate) fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        *self.gamepad_device = gamepad.into();
    }

    /// Returns the [`ActionValue`] for the given [`Input`].
    ///
    /// See also [`Self::consume`] and [`Self::set_gamepad`].
    pub(crate) fn value(&self, input: impl Into<Input>) -> ActionValue {
        let input = input.into();
        match input {
            Input::Keyboard { key, mod_keys } => {
                let pressed = self.action_sources.keyboard
                    && self.keys.pressed(key)
                    && self.mod_keys_pressed(mod_keys)
                    && !self.ignored(input);

                pressed.into()
            }
            Input::MouseButton { button, mod_keys } => {
                let pressed = self.action_sources.mouse_buttons
                    && self.mouse_buttons.pressed(button)
                    && self.mod_keys_pressed(mod_keys)
                    && !self.ignored(input);

                pressed.into()
            }
            Input::MouseMotion { mod_keys } => {
                if !self.action_sources.mouse_motion
                    || !self.mod_keys_pressed(mod_keys)
                    || self.ignored(input)
                {
                    return Vec2::ZERO.into();
                }

                self.mouse_motion.delta.into()
            }
            Input::MouseWheel { mod_keys } => {
                if !self.action_sources.mouse_wheel
                    || !self.mod_keys_pressed(mod_keys)
                    || self.ignored(input)
                {
                    return Vec2::ZERO.into();
                }

                self.mouse_scroll.delta.into()
            }
            Input::GamepadButton(button) => {
                if !self.action_sources.gamepad_button || self.ignored(input) {
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
                if !self.action_sources.gamepad_axis || self.ignored(input) {
                    return 0.0.into();
                }

                let value = match *self.gamepad_device {
                    GamepadDevice::Any => self
                        .gamepads
                        .iter()
                        .filter_map(|gamepad| gamepad.get_unclamped(axis))
                        .reduce(|acc, v| acc + v),
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

        for keys in mod_keys.iter_keys() {
            if !self.keys.any_pressed(keys) {
                return false;
            }
        }

        true
    }

    fn ignored(&self, input: Input) -> bool {
        if *self.skip_ignore_check {
            return false;
        }

        let mut iter = iter::once(&self.pending.ignored).chain(self.consumed.values());
        match input {
            Input::Keyboard { key, mod_keys } => iter
                .any(|inputs| inputs.keys.contains(&key) || inputs.mod_keys.intersects(mod_keys)),
            Input::MouseButton { button, mod_keys } => iter.any(|inputs| {
                inputs.mouse_buttons.contains(&button) || inputs.mod_keys.intersects(mod_keys)
            }),
            Input::MouseMotion { mod_keys } => {
                iter.any(|inputs| inputs.mouse_motion || inputs.mod_keys.intersects(mod_keys))
            }
            Input::MouseWheel { mod_keys } => {
                iter.any(|inputs| inputs.mouse_wheel || inputs.mod_keys.intersects(mod_keys))
            }
            Input::GamepadButton(button) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: button,
                };
                iter.any(|inputs| inputs.gamepad_buttons.contains(&input))
            }
            Input::GamepadAxis(axis) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: axis,
                };
                iter.any(|inputs| inputs.gamepad_axes.contains(&input))
            }
        }
    }

    /// Consumes the input, making it unavailable for [`Self::value`].
    ///
    /// Clears for this schedule with [`Self::clear_consumed`].
    pub(crate) fn consume<S: ScheduleLabel>(&mut self, input: impl Into<Input>) {
        self.consumed
            .entry(TypeId::of::<S>())
            .or_default()
            .add(input.into(), *self.gamepad_device);
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
/// app.add_systems(PreUpdate, disable_mouse.before(EnhancedInputSet::Update));
///
/// fn disable_mouse(
///     mut action_sources: ResMut<ActionSources>,
///     interactions: Query<&Interaction>,
/// ) {
///     let mouse_unused = interactions.iter().all(|&interaction| interaction == Interaction::None);
///     action_sources.mouse_buttons = mouse_unused;
///     action_sources.mouse_wheel = mouse_unused;
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

/// All consumed input by actions in each schedule.
#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct ConsumedInputs(TypeIdMap<IgnoredInputs>);

/// Inputs that will be ignored until they return zero.
///
/// Once the input becomes zero, it will be automatically removed and no longer ignored.
#[derive(Resource, Default)]
pub(crate) struct PendingInputs {
    inputs: Vec<Input>,

    /// Computed from [`Self::inputs`].
    ignored: IgnoredInputs,
}

impl PendingInputs {
    pub(crate) fn extend(&mut self, iter: impl Iterator<Item = Input>) {
        self.inputs
            .extend(iter.inspect(|input| trace!("ignoring '{input}' until reset")));
    }
}

#[derive(Default)]
pub(crate) struct IgnoredInputs {
    keys: HashSet<KeyCode>,
    mod_keys: ModKeys,
    mouse_buttons: HashSet<MouseButton>,
    mouse_motion: bool,
    mouse_wheel: bool,
    gamepad_buttons: HashSet<GamepadInput<GamepadButton>>,
    gamepad_axes: HashSet<GamepadInput<GamepadAxis>>,
}

impl IgnoredInputs {
    fn add(&mut self, input: Input, gamepad: GamepadDevice) {
        match input {
            Input::Keyboard { key, mod_keys } => {
                self.keys.insert(key);
                self.mod_keys.insert(mod_keys);
            }
            Input::MouseButton { button, mod_keys } => {
                self.mouse_buttons.insert(button);
                self.mod_keys.insert(mod_keys);
            }
            Input::MouseMotion { mod_keys } => {
                self.mouse_motion = true;
                self.mod_keys.insert(mod_keys);
            }
            Input::MouseWheel { mod_keys } => {
                self.mouse_wheel = true;
                self.mod_keys.insert(mod_keys);
            }
            Input::GamepadButton(button) => {
                let input = GamepadInput {
                    gamepad,
                    input: button,
                };

                self.gamepad_buttons.insert(input);
            }
            Input::GamepadAxis(axis) => {
                let input = GamepadInput {
                    gamepad,
                    input: axis,
                };

                self.gamepad_axes.insert(input);
            }
        }
    }

    fn clear(&mut self) {
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

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::system::SystemState,
        input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    };

    use super::*;

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

        reader.consume::<PreUpdate>(key);
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

        reader.consume::<PreUpdate>(button);
        assert_eq!(reader.value(button), ActionValue::Bool(false));
    }

    #[test]
    fn mouse_motion() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        world.insert_resource(AccumulatedMouseMotion { delta: value });

        let input = Input::mouse_motion();
        let mut reader = state.get_mut(&mut world);
        reader.clear_consumed::<PreUpdate>();
        assert_eq!(reader.value(input), ActionValue::Axis2D(value));
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SHIFT)),
            ActionValue::Axis2D(Vec2::ZERO)
        );

        reader.consume::<PreUpdate>(input);
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
        reader.clear_consumed::<PreUpdate>();
        assert_eq!(reader.value(input), ActionValue::Axis2D(value));
        assert_eq!(
            reader.value(input.with_mod_keys(ModKeys::SUPER)),
            ActionValue::Axis2D(Vec2::ZERO)
        );

        reader.consume::<PreUpdate>(input);
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

        reader.consume::<PreUpdate>(button1);
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

        reader.consume::<PreUpdate>(button1);
        assert_eq!(reader.value(button1), ActionValue::Axis1D(0.0));

        reader.consume::<PreUpdate>(button2);
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

        reader.consume::<PreUpdate>(axis1);
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

        reader.consume::<PreUpdate>(axis1);
        assert_eq!(reader.value(axis1), ActionValue::Axis1D(0.0));

        reader.consume::<PreUpdate>(axis2);
        assert_eq!(reader.value(axis2), ActionValue::Axis1D(0.0));
    }

    #[test]
    fn any_gamepad_axis_sum() {
        let (mut world, mut state) = init_world();

        let axis = GamepadAxis::LeftStickX;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(axis, 0.001);
        world.spawn(gamepad1);

        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(axis, 0.002);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(axis), ActionValue::Axis1D(0.003));
        assert_eq!(
            reader.value(GamepadAxis::RightStickX),
            ActionValue::Axis1D(0.0)
        );

        reader.consume::<PreUpdate>(axis);
        assert_eq!(reader.value(axis), ActionValue::Axis1D(0.0));
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

        reader.consume::<PreUpdate>(input);
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

        reader.consume::<PreUpdate>(input);
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
        reader.clear_consumed::<PreUpdate>();
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

        reader.consume::<PreUpdate>(input);
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
        reader.clear_consumed::<PreUpdate>();
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

        reader.consume::<PreUpdate>(input);
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
        reader.clear_consumed::<PreUpdate>();

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
        world.init_resource::<ConsumedInputs>();
        world.init_resource::<PendingInputs>();
        world.init_resource::<ActionSources>();

        let state = SystemState::<InputReader>::new(&mut world);

        (world, state)
    }
}
