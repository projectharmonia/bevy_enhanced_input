use bevy::prelude::*;

use crate::{
    input_binding::{BindingBuilder, InputBinding, IntoBindings},
    input_modifier::{negate::Negate, swizzle_axis::SwizzleAxis},
};

/// A preset to map buttons as 2-dimensional input.
///
/// This is a convenience preset that uses [`SwizzleAxis`] and [`Negate`] to
/// bind the buttons to X and Y axes.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space for [`Transform::translation`],
/// you will need to invert Y and apply it to Z inside your observer.
///
/// See also [`Bidirectional`].
///
/// # Examples
///
/// Map keyboard inputs into a 2D movement action
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn binding(
///     trigger: Trigger<Binding<Player>>,
///     settings: Res<KeyboardSettings>,
///     mut players: Query<&mut Actions<Player>>,
/// ) {
///     let mut actions = players.get_mut(trigger.entity()).unwrap();
///     actions.bind::<Move>().to(Cardinal {
///         north: &settings.forward,
///         east: &settings.right,
///         south: &settings.backward,
///         west: &settings.left,
///     });
/// }
///
/// // We use `KeyCode` here because we are only interested in key presses.
/// // But you can also use `Input` if you want to e.g.
/// // combine mouse and keyboard input sources.
/// #[derive(Resource)]
/// struct KeyboardSettings {
///     forward: Vec<KeyCode>,
///     right: Vec<KeyCode>,
///     backward: Vec<KeyCode>,
///     left: Vec<KeyCode>,
/// }
///
/// #[derive(InputContext)]
/// struct Player;
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec2)]
/// struct Move;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Cardinal<I: IntoBindings> {
    pub north: I,
    pub east: I,
    pub south: I,
    pub west: I,
}

impl Cardinal<KeyCode> {
    /// Maps WASD keys as 2-dimensional input.
    ///
    /// See also [`Self::arrow_keys`].
    #[must_use]
    pub fn wasd_keys() -> Self {
        Self {
            north: KeyCode::KeyW,
            west: KeyCode::KeyA,
            south: KeyCode::KeyS,
            east: KeyCode::KeyD,
        }
    }

    /// Maps keyboard arrow keys as 2-dimensional input.
    ///
    /// See also [`Self::wasd_keys`].
    #[must_use]
    pub fn arrow_keys() -> Self {
        Self {
            north: KeyCode::ArrowUp,
            west: KeyCode::ArrowLeft,
            south: KeyCode::ArrowDown,
            east: KeyCode::ArrowRight,
        }
    }
}

impl Cardinal<GamepadButton> {
    /// Maps D-pad as 2-dimensional input.
    ///
    /// See also [`Self::wasd_keys`].
    #[must_use]
    pub fn dpad_buttons() -> Self {
        Self {
            north: GamepadButton::DPadUp,
            west: GamepadButton::DPadLeft,
            south: GamepadButton::DPadDown,
            east: GamepadButton::DPadRight,
        }
    }
}

impl<I: IntoBindings> IntoBindings for Cardinal<I> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        // Y
        let north = self
            .north
            .into_bindings()
            .map(|binding| binding.with_modifiers(SwizzleAxis::YXZ));

        // -X
        let west = self
            .west
            .into_bindings()
            .map(|binding| binding.with_modifiers(Negate::all()));

        // -Y
        let south = self
            .south
            .into_bindings()
            .map(|binding| binding.with_modifiers((Negate::all(), SwizzleAxis::YXZ)));

        // X
        let east = self.east.into_bindings();

        north.chain(east).chain(south).chain(west)
    }
}

/// A preset to map buttons as 1-dimensional input.
///
/// Positive binding will be passed as is and negative will be reversed using [`Negate`].
///
/// See also [`Cardinal`].
#[derive(Debug, Clone, Copy)]
pub struct Bidirectional<I: IntoBindings> {
    pub positive: I,
    pub negative: I,
}

impl<I: IntoBindings> IntoBindings for Bidirectional<I> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        let positive = self.positive.into_bindings();
        let negative = self
            .negative
            .into_bindings()
            .map(|binding| binding.with_modifiers(Negate::all()));

        positive.chain(negative)
    }
}

/// A preset to map a stick as 1-dimensional input.
///
/// Represents the side of a gamepad's analog stick.
#[derive(Debug, Clone, Copy)]
pub enum GamepadStick {
    /// Corresponds to [`GamepadAxis::LeftStickX`] and [`GamepadAxis::LeftStickY`]
    Left,
    /// Corresponds to [`GamepadAxis::RightStickX`] and [`GamepadAxis::RightStickY`]
    Right,
}

impl GamepadStick {
    /// Returns associated X axis.
    pub fn x(self) -> GamepadAxis {
        match self {
            GamepadStick::Left => GamepadAxis::LeftStickX,
            GamepadStick::Right => GamepadAxis::RightStickX,
        }
    }

    /// Returns associated Y axis.
    pub fn y(self) -> GamepadAxis {
        match self {
            GamepadStick::Left => GamepadAxis::LeftStickY,
            GamepadStick::Right => GamepadAxis::RightStickY,
        }
    }
}

impl IntoBindings for GamepadStick {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        [self.x().into(), self.y().with_modifiers(SwizzleAxis::YXZ)].into_iter()
    }
}
