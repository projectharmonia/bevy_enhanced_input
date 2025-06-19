use alloc::boxed::Box;

use bevy::utils::TypeIdMap;
use log::trace;

use crate::prelude::*;

/// Helper to calculate [`ActionState`] based on its modifiers and conditions.
///
/// Could be used to track both input-level state and action-level state.
pub(super) struct TriggerTracker {
    value: ActionValue,
    found_explicit: bool,
    any_explicit_fired: bool,
    found_active: bool,
    found_implicit: bool,
    all_implicits_fired: bool,
    blocked: bool,
}

impl TriggerTracker {
    #[must_use]
    pub(super) fn new(value: ActionValue) -> Self {
        Self {
            value,
            found_explicit: false,
            any_explicit_fired: false,
            found_active: false,
            found_implicit: false,
            all_implicits_fired: true,
            blocked: false,
        }
    }

    pub(super) fn apply_modifiers(
        &mut self,
        action_map: &TypeIdMap<UntypedAction>,
        time: &InputTime,
        modifiers: &mut [Box<dyn InputModifier>],
    ) {
        for modifier in modifiers {
            let new_value = modifier.apply(action_map, time, self.value);
            trace!(
                "`{modifier:?}` changes `{:?}` to `{new_value:?}`",
                self.value
            );

            self.value = new_value;
        }
    }

    pub(super) fn apply_conditions(
        &mut self,
        action_map: &TypeIdMap<UntypedAction>,
        time: &InputTime,
        conditions: &mut [Box<dyn InputCondition>],
    ) {
        // Note: No early outs permitted!
        // All conditions must be evaluated to update their internal state/delta time.
        for condition in conditions {
            let state = condition.evaluate(action_map, time, self.value);
            trace!("`{condition:?}` returns state `{state:?}`");
            match condition.kind() {
                ConditionKind::Explicit => {
                    self.found_explicit = true;
                    self.any_explicit_fired |= state == ActionState::Fired;
                    self.found_active |= state != ActionState::None;
                }
                ConditionKind::Implicit => {
                    self.found_implicit = true;
                    self.all_implicits_fired &= state == ActionState::Fired;
                    self.found_active |= state != ActionState::None;
                }
                ConditionKind::Blocker => {
                    self.blocked |= state == ActionState::None;
                }
            }
        }
    }

    pub(super) fn state(&self) -> ActionState {
        if self.blocked {
            return ActionState::None;
        }

        if !self.found_explicit && !self.found_implicit {
            if self.value.as_bool() {
                return ActionState::Fired;
            } else {
                return ActionState::None;
            }
        }

        if (!self.found_explicit || self.any_explicit_fired) && self.all_implicits_fired {
            ActionState::Fired
        } else if self.found_active {
            ActionState::Ongoing
        } else {
            ActionState::None
        }
    }

    pub(super) fn value(&self) -> ActionValue {
        self.value
    }

    /// Replaces the state with `other`.
    ///
    /// Preserves the value dimension.
    pub(super) fn overwrite(&mut self, other: TriggerTracker) {
        let dim = self.value.dim();
        *self = other;
        self.value = self.value.convert(dim);
    }

    /// Merges two trackers.
    ///
    /// Preserves the value dimension.
    pub(super) fn combine(&mut self, other: Self, accumulation: Accumulation) {
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
        self.found_explicit |= other.found_explicit;
        self.any_explicit_fired |= other.any_explicit_fired;
        self.found_active |= other.found_active;
        self.found_implicit |= other.found_implicit;
        self.all_implicits_fired &= other.all_implicits_fired;
        self.blocked |= other.blocked;
    }
}
