use bevy::prelude::*;

use super::{
    input_bind::{InputBind, InputBindModCond},
    input_modifier::{negate::Negate, swizzle_axis::SwizzleAxis},
};

pub trait BindPreset {
    fn bindings(self) -> impl Iterator<Item = InputBind>;
}

impl<I: Into<InputBind>> BindPreset for I {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        std::iter::once(self.into())
    }
}

impl<T: Into<InputBind>, const N: usize> BindPreset for [T; N] {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.into_iter().map(Into::into)
    }
}

/// Maps WASD keys as 2-dimentional input.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space, you will
/// need to invert Y and apply it to Z translation inside your observer.
///
/// Shorthand for [`Cardinal`].
///
/// See also [`ArrowKeys`].
#[derive(Debug, Clone, Copy, Default)]
pub struct WasdKeys;

impl BindPreset for WasdKeys {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        Cardinal {
            north: KeyCode::KeyW,
            east: KeyCode::KeyA,
            south: KeyCode::KeyS,
            west: KeyCode::KeyD,
        }
        .bindings()
    }
}

/// Maps keyboard arrow keys as 2-dimentional input.
///
/// Shorthand for [`Cardinal`].
///
/// See also [`WasdKeys`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ArrowKeys;

impl BindPreset for ArrowKeys {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        Cardinal {
            north: KeyCode::ArrowUp,
            east: KeyCode::ArrowLeft,
            south: KeyCode::ArrowDown,
            west: KeyCode::ArrowRight,
        }
        .bindings()
    }
}

/// Maps D-pad as 2-dimentional input.
///
/// Shorthand for [`Cardinal`].
///
/// See also [`WasdKeys`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DpadButtons;

impl BindPreset for DpadButtons {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        Cardinal {
            north: GamepadButtonType::DPadUp,
            east: GamepadButtonType::DPadLeft,
            south: GamepadButtonType::DPadDown,
            west: GamepadButtonType::DPadRight,
        }
        .bindings()
    }
}

/// Maps 4 buttons as 2-dimentional input.
///
/// This is a convenience preset that uses [`SwizzleAxis`] and [`Negate`] to
/// bind the buttons to X and Y axes.
#[derive(Debug, Clone, Copy)]
pub struct Cardinal<N, E, S, W> {
    pub north: N,
    pub east: E,
    pub south: S,
    pub west: W,
}

impl<N, E, S, W> BindPreset for Cardinal<N, E, S, W>
where
    N: Into<InputBind>,
    E: Into<InputBind>,
    S: Into<InputBind>,
    W: Into<InputBind>,
{
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        [
            self.north.with_modifier(SwizzleAxis::YXZ),
            self.east.with_modifier(Negate::default()),
            self.south
                .with_modifier(Negate::default())
                .with_modifier(SwizzleAxis::YXZ),
            self.west.into(),
        ]
        .into_iter()
    }
}

/// Represents the side of a gamepad's analog stick.
#[derive(Clone, Copy, Debug)]
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
