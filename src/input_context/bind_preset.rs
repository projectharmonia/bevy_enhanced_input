use std::iter;

use bevy::prelude::*;

use super::{
    input_bind::{InputBind, InputBindModCond},
    input_modifier::{negate::Negate, swizzle_axis::SwizzleAxis},
};
use crate::input::Input;

pub trait BindPreset {
    fn bindings(self) -> impl Iterator<Item = InputBind>;
}

impl<I: Into<InputBind>> BindPreset for I {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        iter::once(self.into())
    }
}

impl<I: Into<InputBind> + Copy> BindPreset for &Vec<I> {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.as_slice().bindings()
    }
}

impl<I: Into<InputBind> + Copy, const N: usize> BindPreset for &[I; N] {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.as_slice().bindings()
    }
}

impl<I: Into<InputBind> + Copy> BindPreset for &[I] {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.iter().copied().map(Into::into)
    }
}

/// A preset to map buttons as 2-dimentional input.
///
/// This is a convenience preset that uses [`SwizzleAxis`] and [`Negate`] to
/// bind the buttons to X and Y axes.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space for [`Transform::translation`],
/// you will need to invert Y and apply it to Z inside your observer.
///
/// # Examples
///
/// Map keyboard inputs into a 2D movement action
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// // We use `KeyCode` here because we are only interested in key presses.
/// // But you can also use `Input` if you want to e.g.
/// // combine mouse and keyboard input sources.
/// #[derive(Debug, Resource)]
/// struct KeyboardSettings {
///     forward: Vec<KeyCode>,
///     right: Vec<KeyCode>,
///     backward: Vec<KeyCode>,
///     left: Vec<KeyCode>,
/// }
///
/// #[derive(Debug, Component)]
/// struct Player;
///
/// impl InputContext for Player {
///     fn context_instance(world: &World, _entity: Entity) -> ContextInstance {
///         let settings = world.resource::<KeyboardSettings>();
///
///         let mut ctx = ContextInstance::default();
///         ctx.bind::<Move>().to(Cardinal {
///             north: &settings.forward,
///             east: &settings.right,
///             south: &settings.backward,
///             west: &settings.left,
///         });
///         ctx
///     }
/// }
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec2)]
/// struct Move;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Cardinal<'a, I> {
    pub north: &'a [I],
    pub east: &'a [I],
    pub south: &'a [I],
    pub west: &'a [I],
}

impl Cardinal<'_, KeyCode> {
    /// Maps WASD keys as 2-dimentional input.
    ///
    /// See also [`Self::arrow_keys`].
    #[must_use]
    pub fn wasd_keys() -> Self {
        Self {
            north: &[KeyCode::KeyW],
            east: &[KeyCode::KeyA],
            south: &[KeyCode::KeyS],
            west: &[KeyCode::KeyD],
        }
    }

    /// Maps keyboard arrow keys as 2-dimentional input.
    ///
    /// See also [`Self::wasd_keys`].
    #[must_use]
    pub fn arrow_keys() -> Self {
        Self {
            north: &[KeyCode::ArrowUp],
            east: &[KeyCode::ArrowLeft],
            south: &[KeyCode::ArrowDown],
            west: &[KeyCode::ArrowRight],
        }
    }
}

impl Cardinal<'_, GamepadButtonType> {
    /// Maps D-pad as 2-dimentional input.
    ///
    /// See also [`Self::wasd_keys`].
    #[must_use]
    pub fn dpad_buttons() -> Self {
        Self {
            north: &[GamepadButtonType::DPadUp],
            east: &[GamepadButtonType::DPadLeft],
            south: &[GamepadButtonType::DPadDown],
            west: &[GamepadButtonType::DPadRight],
        }
    }
}

impl<I: Into<Input> + InputBindModCond + Copy> BindPreset for Cardinal<'_, I> {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        // Y
        let north = self
            .north
            .iter()
            .map(|binding| binding.with_modifier(SwizzleAxis::YXZ));

        // -X
        let east = self
            .east
            .iter()
            .map(|binding| binding.with_modifier(Negate::default()));

        // -Y
        let south = self.south.iter().map(|binding| {
            binding
                .with_modifier(Negate::default())
                .with_modifier(SwizzleAxis::YXZ)
        });

        // X
        let west = self.west.iter().copied().map(Into::into).map(Into::into);

        north.chain(east).chain(south).chain(west)
    }
}

/// A preset to map a stick as 2-dimentional input.
///
/// Represents the side of a gamepad's analog stick.
#[derive(Debug, Clone, Copy)]
pub enum GamepadStick {
    /// Corresponds to [`GamepadAxisType::LeftStickX`] and [`GamepadAxisType::LeftStickY`]
    Left,
    /// Corresponds to [`GamepadAxisType::RightStickX`] and [`GamepadAxisType::RightStickY`]
    Right,
}

impl GamepadStick {
    /// Returns associated X axis.
    pub fn x(self) -> GamepadAxisType {
        match self {
            GamepadStick::Left => GamepadAxisType::LeftStickX,
            GamepadStick::Right => GamepadAxisType::RightStickX,
        }
    }

    /// Returns associated Y axis.
    pub fn y(self) -> GamepadAxisType {
        match self {
            GamepadStick::Left => GamepadAxisType::LeftStickY,
            GamepadStick::Right => GamepadAxisType::RightStickY,
        }
    }
}

impl BindPreset for GamepadStick {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        [self.x().into(), self.y().with_modifier(SwizzleAxis::YXZ)].into_iter()
    }
}
