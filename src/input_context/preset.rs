use bevy::prelude::*;

use crate::{Negate, SwizzleAxis};

use super::bind::{BindConfigs, IntoBindConfigs};

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

impl<N, E, S, W> IntoBindConfigs for Cardinal<N, E, S, W>
where
    N: IntoBindConfigs,
    E: IntoBindConfigs,
    S: IntoBindConfigs,
    W: IntoBindConfigs,
{
    fn into_configs(self) -> BindConfigs {
        (
            self.north.with_modifier(SwizzleAxis::YXZ),
            self.east,
            self.south
                .with_modifier(Negate::default())
                .with_modifier(SwizzleAxis::YXZ),
            self.west.with_modifier(Negate::default()),
        )
            .into_configs()
    }
}

/// Maps WASD keys as 2-dimentional input.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space, you will
/// need to invert Y and apply it to Z translation inside your observer.
///
/// Shorthand for [`Cardinal`] with [`KeyCode`] WASD keys.
///
/// See also [`ArrowKeys`], [`DPadButtons`].
#[derive(Debug, Clone, Copy, Default)]
pub struct WasdKeys;

impl IntoBindConfigs for WasdKeys {
    fn into_configs(self) -> BindConfigs {
        Cardinal {
            north: KeyCode::KeyW,
            east: KeyCode::KeyD,
            south: KeyCode::KeyS,
            west: KeyCode::KeyA,
        }
        .into_configs()
    }
}

/// Maps keyboard arrow keys as 2-dimentional input.
///
/// Shorthand for [`Cardinal`] with [`KeyCode`] arrow keys.
///
/// See also [`WasdKeys`], [`DPadButtons`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ArrowKeys;

impl IntoBindConfigs for ArrowKeys {
    fn into_configs(self) -> BindConfigs {
        Cardinal {
            north: KeyCode::ArrowUp,
            east: KeyCode::ArrowRight,
            south: KeyCode::ArrowDown,
            west: KeyCode::ArrowLeft,
        }
        .into_configs()
    }
}

/// Maps D-pad as 2-dimentional input.
///
/// Shorthand for [`Cardinal`] with [`GamepadButtonType`] D-pad keys.
///
/// See also [`WasdKeys`], [`ArrowKeys`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DPadButtons;

impl IntoBindConfigs for DPadButtons {
    fn into_configs(self) -> BindConfigs {
        Cardinal {
            north: GamepadButtonType::DPadUp,
            east: GamepadButtonType::DPadRight,
            south: GamepadButtonType::DPadDown,
            west: GamepadButtonType::DPadLeft,
        }
        .into_configs()
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

impl IntoBindConfigs for GamepadStick {
    fn into_configs(self) -> BindConfigs {
        (self.x(), self.y().with_modifier(SwizzleAxis::YXZ)).into_configs()
    }
}
