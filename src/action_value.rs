use bevy::{input::ButtonState, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum ActionValue {
    Bool(bool),
    Axis1D(f32),
    Axis2D(Vec2),
    Axis3D(Vec3),
}

impl ActionValue {
    pub fn zero(dim: ActionValueDim) -> Self {
        match dim {
            ActionValueDim::Bool => ActionValue::Bool(false),
            ActionValueDim::Axis1D => ActionValue::Axis1D(0.0),
            ActionValueDim::Axis2D => ActionValue::Axis2D(Vec2::ZERO),
            ActionValueDim::Axis3D => ActionValue::Axis3D(Vec3::ZERO),
        }
    }

    pub fn dim(self) -> ActionValueDim {
        match self {
            Self::Bool(_) => ActionValueDim::Bool,
            Self::Axis1D(_) => ActionValueDim::Axis1D,
            Self::Axis2D(_) => ActionValueDim::Axis2D,
            Self::Axis3D(_) => ActionValueDim::Axis3D,
        }
    }

    pub fn convert(self, dim: ActionValueDim) -> Self {
        match dim {
            ActionValueDim::Bool => self.as_bool().into(),
            ActionValueDim::Axis1D => self.as_axis1d().into(),
            ActionValueDim::Axis2D => self.as_axis2d().into(),
            ActionValueDim::Axis3D => self.as_axis3d().into(),
        }
    }

    pub fn as_bool(self) -> bool {
        match self {
            Self::Bool(value) => value,
            Self::Axis1D(value) => value != 0.0,
            Self::Axis2D(value) => value != Vec2::ZERO,
            Self::Axis3D(value) => value != Vec3::ZERO,
        }
    }

    pub fn as_axis1d(self) -> f32 {
        match self {
            Self::Bool(value) => {
                if value {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Axis1D(value) => value,
            Self::Axis2D(value) => value.x,
            Self::Axis3D(value) => value.x,
        }
    }

    pub fn as_axis2d(self) -> Vec2 {
        match self {
            Self::Bool(value) => {
                if value {
                    Vec2::X
                } else {
                    Vec2::ZERO
                }
            }
            Self::Axis1D(value) => Vec2::X * value,
            Self::Axis2D(value) => value,
            Self::Axis3D(value) => value.xy(),
        }
    }

    pub fn as_axis3d(self) -> Vec3 {
        match self {
            Self::Bool(value) => {
                if value {
                    Vec3::X
                } else {
                    Vec3::ZERO
                }
            }
            Self::Axis1D(value) => Vec3::X * value,
            Self::Axis2D(value) => value.extend(0.0),
            Self::Axis3D(value) => value,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ActionValueDim {
    Bool,
    Axis1D,
    Axis2D,
    Axis3D,
}

impl From<bool> for ActionValue {
    fn from(value: bool) -> Self {
        ActionValue::Bool(value)
    }
}

impl From<f32> for ActionValue {
    fn from(value: f32) -> Self {
        ActionValue::Axis1D(value)
    }
}

impl From<Vec2> for ActionValue {
    fn from(value: Vec2) -> Self {
        ActionValue::Axis2D(value)
    }
}

impl From<Vec3> for ActionValue {
    fn from(value: Vec3) -> Self {
        ActionValue::Axis3D(value)
    }
}

impl From<ButtonState> for ActionValue {
    fn from(value: ButtonState) -> Self {
        match value {
            ButtonState::Pressed => true.into(),
            ButtonState::Released => false.into(),
        }
    }
}
