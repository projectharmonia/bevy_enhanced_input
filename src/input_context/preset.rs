use bevy::prelude::*;

use crate::{Input, InputBind, InputCondition, InputModifier, Negate, SwizzleAxis};

pub trait BindPreset {
    fn bindings(self) -> impl Iterator<Item = InputBind>;

    fn with_modifier<M: InputModifier + Clone>(self, modifier: M) -> PresetWithModifier<Self, M>
    where
        Self: Sized,
    {
        PresetWithModifier {
            preset: self,
            modifier,
        }
    }

    fn with_condition<C: InputCondition + Clone>(self, condition: C) -> PresetWithCondition<Self, C>
    where
        Self: Sized,
    {
        PresetWithCondition {
            preset: self,
            condition,
        }
    }
}

impl<I: Into<InputBind>> BindPreset for I {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        std::iter::once(self.into())
    }
}

pub struct PresetWithModifier<P, M> {
    preset: P,
    modifier: M,
}

impl<P: BindPreset, M: InputModifier + Clone> BindPreset for PresetWithModifier<P, M> {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.preset
            .bindings()
            .map(move |bind| bind.with_modifier(self.modifier.clone()))
    }
}

pub struct PresetWithCondition<P, C> {
    preset: P,
    condition: C,
}

impl<P: BindPreset, C: InputCondition + Clone> BindPreset for PresetWithCondition<P, C> {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        self.preset
            .bindings()
            .map(move |bind| bind.with_condition(self.condition.clone()))
    }
}

/// Maps 4 buttons as 2-dimentional input.
///
/// This is a convenience preset that uses [`SwizzleAxis`] and [`Negate`] to
/// bind the buttons to X and Y axes.
#[derive(Debug, Clone, Copy)]
pub struct XyAxis {
    pub up: Input,
    pub left: Input,
    pub down: Input,
    pub right: Input,
}

impl BindPreset for XyAxis {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        [
            InputBind::new(self.up).with_modifier(SwizzleAxis::YXZ),
            InputBind::new(self.left).with_modifier(Negate::default()),
            InputBind::new(self.down)
                .with_modifier(Negate::default())
                .with_modifier(SwizzleAxis::YXZ),
            InputBind::new(self.right),
        ]
        .into_iter()
    }
}

/// Maps WASD keys as 2-dimentional input.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space, you will
/// need to invert Y and apply it to Z translation inside your observer.
///
/// Shorthand for [`XyAxis`].
///
/// See also [`ArrowKeys`].
#[derive(Debug, Clone, Copy, Default)]
pub struct WasdKeys;

impl BindPreset for WasdKeys {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        XyAxis {
            up: KeyCode::KeyW.into(),
            left: KeyCode::KeyA.into(),
            down: KeyCode::KeyS.into(),
            right: KeyCode::KeyD.into(),
        }
        .bindings()
    }
}

/// Maps keyboard arrow keys as 2-dimentional input.
///
/// Shorthand for [`XyAxis`].
///
/// See also [`WasdKeys`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ArrowKeys;

impl BindPreset for ArrowKeys {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        XyAxis {
            up: KeyCode::ArrowUp.into(),
            left: KeyCode::ArrowLeft.into(),
            down: KeyCode::ArrowDown.into(),
            right: KeyCode::ArrowRight.into(),
        }
        .bindings()
    }
}

/// Maps D-pad as 2-dimentional input.
///
/// Shorthand for [`XyAxis`].
///
/// See also [`WasdKeys`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DpadButtons;

impl BindPreset for DpadButtons {
    fn bindings(self) -> impl Iterator<Item = InputBind> {
        XyAxis {
            up: GamepadButtonType::DPadUp.into(),
            left: GamepadButtonType::DPadLeft.into(),
            down: GamepadButtonType::DPadDown.into(),
            right: GamepadButtonType::DPadRight.into(),
        }
        .bindings()
    }
}
