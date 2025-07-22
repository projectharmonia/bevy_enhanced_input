use crate::prelude::*;
use bevy::{prelude::*, utils::TypeIdMap};

#[derive(Clone, Copy, Debug)]
pub struct LinearAccelerate {
    pub step_rate: f32,
    current_value: Vec3,
}

impl LinearAccelerate {
    #[must_use]
    pub fn new(step_rate: f32) -> Self {
        Self {
            step_rate,
            current_value: Default::default(),
        }
    }
}

impl Default for LinearAccelerate {
    fn default() -> Self {
        Self::new(0.1)
    }
}

impl InputModifier for LinearAccelerate {
    fn apply(
        &mut self,
        _action_map: &TypeIdMap<UntypedAction>,
        _time: &InputTime,
        value: ActionValue,
    ) -> ActionValue {
        let target_value = value.as_axis3d();
        if (0.0..self.step_rate).contains(&self.current_value.distance_squared(target_value)) {
            self.current_value = target_value;
            return value;
        }
        let difference = target_value.length() - self.current_value.length();
        match difference {
            0.0.. => self.current_value += self.step_rate * target_value,
            ..0.0 => self.current_value -= self.step_rate * self.current_value.signum(),
            _ => {}
        }
        ActionValue::Axis3D(self.current_value).convert(value.dim())
    }
}
