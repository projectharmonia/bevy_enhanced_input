use std::cmp::Ordering;

use bevy::prelude::*;

use super::{
    input_action::{Accumulation, ActionState, ActionsData},
    input_condition::{ConditionKind, InputCondition},
    input_modifier::InputModifier,
};
use crate::action_value::ActionValue;

/// Helper to calculate [`ActionState`] based on its modifiers and conditions.
///
/// Could be used to track both input-level state and action-level state.
pub(super) struct TriggerTracker {
    value: ActionValue,
    state: ActionState,
    blocked: bool,
    found_explicit: bool,
}

impl TriggerTracker {
    #[must_use]
    pub(super) fn new(value: ActionValue) -> Self {
        Self {
            value,
            state: Default::default(),
            blocked: false,
            found_explicit: false,
        }
    }

    pub(super) fn apply_modifiers(
        &mut self,
        world: &World,
        delta: f32,
        modifiers: &mut [Box<dyn InputModifier>],
    ) {
        for modifier in modifiers {
            let new_value = modifier.apply(world, delta, self.value);
            debug_assert_eq!(
                new_value.dim(),
                self.value.dim(),
                "modifiers should preserve action dimentions"
            );
            trace!(
                "`{modifier:?}` changes `{:?}` to `{new_value:?}`",
                self.value
            );

            self.value = new_value;
        }
    }

    pub(super) fn apply_conditions(
        &mut self,
        world: &World,
        actions_data: &ActionsData,
        delta: f32,
        conditions: &mut [Box<dyn InputCondition>],
    ) {
        // Note: No early outs permitted!
        // All conditions must be evaluated to update their internal state/delta time.
        for condition in conditions {
            let state = condition.evaluate(world, actions_data, delta, self.value);
            trace!("`{condition:?}` returns state `{state:?}`");
            match condition.kind() {
                ConditionKind::Explicit => {
                    self.found_explicit = true;
                    if state > self.state {
                        // Retain the most interesting.
                        self.state = state;
                    }
                }
                ConditionKind::Implicit => {
                    if state != ActionState::Fired {
                        self.blocked = true;
                    }
                }
            }
        }
    }

    /// Merges input-level tracker into an action-level tracker.
    pub(super) fn merge(&mut self, other: Self, accumulation: Accumulation) {
        if other.blocked {
            // Input-level tracker that are blocked by a condition
            // shouldn't affection action-level trackers.
            return;
        }

        if other.found_explicit {
            self.found_explicit = true;
        }

        match self.state.cmp(&other.state) {
            Ordering::Less => {
                self.state = other.state;
                self.value = other.value;
            }
            Ordering::Equal => {
                let accumulated = match accumulation {
                    Accumulation::MaxAbs => {
                        let mut value = self.value.as_axis3d().to_array();
                        let other_value = other.value.as_axis3d().to_array();
                        for (axis, other_axis) in value.iter_mut().zip(other_value) {
                            if axis.abs() < other_axis.abs() {
                                *axis = other_axis;
                            }
                        }
                        value.into()
                    }
                    Accumulation::Cumulative => self.value.as_axis3d() + other.value.as_axis3d(),
                };
                self.value = ActionValue::Axis3D(accumulated).convert(self.value.dim());
            }
            Ordering::Greater => (),
        }
    }

    pub(super) fn finish(mut self) -> (ActionState, ActionValue) {
        if self.blocked {
            self.state = ActionState::None
        } else if !self.found_explicit && self.value.as_bool() {
            self.state = ActionState::Fired;
        }

        (self.state, self.value)
    }
}
