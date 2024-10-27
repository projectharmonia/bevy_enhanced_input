use std::any::{self, TypeId};

use bevy::prelude::*;

use super::{condition_timer::ConditionTimer, ConditionKind, InputCondition};
use crate::{
    action_value::ActionValue,
    input_context::input_action::{ActionEvents, ActionState, ActionsData, InputAction},
};

/// Actions that needs to be pressed in sequence.
///
/// All actions in [`Self::steps`] must be completed in the defined order to activate this condition.
/// The condition activates for one frame before resetting the combo progress.
#[derive(Debug, Clone, Default)]
pub struct Combo {
    /// Tracks of what action we're currently at in the combo.
    step_index: usize,

    /// Time elapsed between last combo step and the current time.
    timer: ConditionTimer,

    /// List of input actions that need to be completed (according to [`ComboStep::events`]) to activate this action.
    ///
    /// Input actions must be triggered in order (starting at index 0) to count towards the triggering of the combo.
    pub steps: Vec<ComboStep>,

    /// Actions that will cancel the combo if they are completed (according to [`CancelAction::events`]).
    pub cancel_actions: CancelActions,
}

impl Combo {
    /// Like [`Self::step_with`], but uses [`ActionEvents::FIRED`] for events.
    pub fn step<A: InputAction>(self, trigger_time: f32) -> Self {
        self.step_with::<A>(trigger_time, ActionEvents::FIRED)
    }

    /// Adds a step to the combo sequence with a specific time to trigger any of the events.
    ///
    /// See also [`Self::step`].
    pub fn step_with<A: InputAction>(mut self, trigger_time: f32, events: ActionEvents) -> Self {
        let step = ComboStep::new::<A>(trigger_time, events);
        self.steps.push(step);
        self
    }

    /// Cancel the combo if any other action in the same context (excluding the current step action) triggers any of the events.
    pub fn any_cancel_action(mut self, events: ActionEvents) -> Self {
        self.cancel_actions = CancelActions::Any(events);
        self
    }

    /// Like [`Self::cancel_action_with`], but uses [`ActionEvents::FIRED`] for events.
    pub fn cancel_action<A: InputAction>(self) -> Self {
        self.cancel_action_with::<A>(ActionEvents::FIRED)
    }

    /// Cancel the combo if action `A` in the same context triggers any of the events.
    ///
    /// See also [`Self::cancel_action`].
    pub fn cancel_action_with<A: InputAction>(mut self, events: ActionEvents) -> Self {
        let action = CancelAction::new::<A>(events);
        match &mut self.cancel_actions {
            CancelActions::Any(_) => self.cancel_actions = CancelActions::Actions(vec![action]),
            CancelActions::Actions(cancel_actions) => {
                cancel_actions.push(CancelAction::new::<A>(events))
            }
        }

        self
    }

    fn cancel(&mut self) {
        self.step_index = 0;
        self.timer.reset();
    }

    fn cancelled(&self, actions_data: &ActionsData) -> bool {
        let current_step = self.steps[self.step_index];
        match &self.cancel_actions {
            CancelActions::Actions(cancel_actions) => {
                for cancel_action in cancel_actions {
                    if cancel_action.type_id == current_step.type_id {
                        continue;
                    }
                    let Some(action) = actions_data.get(&cancel_action.type_id) else {
                        warn_once!(
                            "cancel action `{}` is not present in context",
                            cancel_action.name,
                        );
                        continue;
                    };

                    if action.events().intersects(cancel_action.events) {
                        return true;
                    }
                }

                // Check if any other step is also triggered, breaking the order.
                for step in &self.steps {
                    if step.type_id == current_step.type_id {
                        continue;
                    }
                    let Some(action) = actions_data.get(&step.type_id) else {
                        warn_once!("step action `{}` is not present in context", step.name);
                        continue;
                    };

                    if action.events().intersects(step.events) {
                        return true;
                    }
                }
            }
            CancelActions::Any(events) => {
                for (&type_id, action) in actions_data.iter() {
                    if type_id == current_step.type_id {
                        continue;
                    }

                    if action.events().intersects(*events) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

impl InputCondition for Combo {
    fn evaluate(
        &mut self,
        actions_data: &ActionsData,
        timer: &Time<Virtual>,
        _value: ActionValue,
    ) -> ActionState {
        if self.steps.is_empty() {
            warn_once!("combo `{}` has no combo steps", any::type_name::<Self>());
            return ActionState::None;
        }

        if self.cancelled(actions_data) {
            // We don't early-return since the first step could be triggered.
            self.cancel();
        }

        if self.step_index > 0 {
            self.timer.update(timer);

            if self.timer.duration() > self.steps[self.step_index].trigger_time {
                self.cancel();
            }
        }

        let current_step = self.steps[self.step_index];
        let Some(current_action) = actions_data.get(&current_step.type_id) else {
            warn_once!(
                "step action `{}` is not present in context",
                current_step.name
            );
            self.cancel();
            return ActionState::None;
        };

        if current_action.events().intersects(current_step.events) {
            self.step_index += 1;
            self.timer.reset();

            if self.step_index >= self.steps.len() {
                // Completed all combo actions.
                self.step_index = 0;
                return ActionState::Fired;
            }
        }

        if self.step_index > 0 || current_action.state() > ActionState::None {
            return ActionState::Ongoing;
        }

        self.timer.reset();
        ActionState::None
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Required
    }
}

/// Actions and events that cancel [`Combo`].
#[derive(Debug, Clone)]
pub enum CancelActions {
    /// Cancel the combo if any action (excluding the current step) triggers any of this events.
    Any(ActionEvents),
    /// Cancel the combo if any listed action (excluding the current step) triggers associated events.
    Actions(Vec<CancelAction>),
}

impl Default for CancelActions {
    fn default() -> Self {
        Self::Any(ActionEvents::FIRED)
    }
}

/// Action and events that progress [`Combo`].
#[derive(Debug, Clone, Copy)]
pub struct ComboStep {
    /// Associated action.
    type_id: TypeId,

    /// Action display name.
    name: &'static str,

    // Action events for the action to complete this step.
    pub events: ActionEvents,

    /// Time to trigger [`Self::events`] before the combo is cancelled.
    ///
    /// Starts once the previous step in the combo is completed.
    /// This can be safely ignored for the first action in the combo.
    pub trigger_time: f32,
}

impl ComboStep {
    pub fn new<A: InputAction>(trigger_time: f32, events: ActionEvents) -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            name: any::type_name::<A>(),
            events,
            trigger_time,
        }
    }
}

/// Action and events that cancel [`Combo`].
#[derive(Debug, Clone, Copy)]
pub struct CancelAction {
    /// Associated action.
    type_id: TypeId,

    /// Action display name.
    name: &'static str,

    // Action events for this action that will cancel the combo
    pub events: ActionEvents,
}

impl CancelAction {
    pub fn new<A: InputAction>(events: ActionEvents) -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            name: any::type_name::<A>(),
            events,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::{
        action_value::ActionValueDim,
        input_context::input_action::{ActionData, ActionsData},
    };

    #[test]
    fn empty() {
        let mut condition = Combo::default();
        let time = Time::default();
        let actions = ActionsData::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn missing_first_step() {
        let mut condition = Combo::default().step::<ActionA>(0.5);
        let time = Time::default();
        let actions = ActionsData::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn missing_other_step() {
        let mut condition = Combo::default().step::<ActionA>(0.5).step::<ActionB>(0.5);
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );

        set_action::<ActionA>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn timeout() {
        let mut condition = Combo::default().step::<ActionA>(0.5).step::<ActionB>(0.5);
        let mut time = Time::default();
        time.advance_by(Duration::from_secs(1));
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionB>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );

        time.advance_by(Duration::from_secs(1));
        set_action::<ActionA>(&time, &mut actions, ActionState::None);
        set_action::<ActionB>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn first_step_ongoing() {
        let mut condition = Combo::default().step::<ActionA>(0.5);
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Ongoing);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
    }

    #[test]
    fn steps() {
        let mut condition = Combo::default().step::<ActionA>(0.5).step::<ActionB>(0.5);
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionB>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );

        set_action::<ActionA>(&time, &mut actions, ActionState::None);
        set_action::<ActionB>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn out_of_order() {
        let mut condition = Combo::default()
            .step::<ActionA>(0.5)
            .step::<ActionB>(0.5)
            .step::<ActionC>(0.5)
            .cancel_action::<ActionD>(); // Out of order is relevant only for non-any cancel actions.
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::None);
        set_action::<ActionB>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionC>(&time, &mut actions, ActionState::None);
        set_action::<ActionD>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );

        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionB>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );

        set_action::<ActionA>(&time, &mut actions, ActionState::None);
        set_action::<ActionC>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn ignore_same_cancel_action() {
        let mut condition = Combo::default()
            .step::<ActionA>(0.5)
            .cancel_action::<ActionA>();
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
    }

    #[test]
    fn missing_cancel_action() {
        let mut condition = Combo::default()
            .step::<ActionA>(0.5)
            .cancel_action::<ActionA>()
            .cancel_action::<ActionB>();
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
    }

    #[test]
    fn cancel_action() {
        let mut condition = Combo::default()
            .step::<ActionA>(0.5)
            .step::<ActionB>(0.5)
            .cancel_action::<ActionC>();
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionB>(&time, &mut actions, ActionState::None);
        set_action::<ActionC>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );

        set_action::<ActionA>(&time, &mut actions, ActionState::None);
        set_action::<ActionB>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionC>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn any_cancel_action() {
        let mut condition = Combo::default()
            .step::<ActionA>(0.5)
            .step::<ActionB>(0.5)
            .any_cancel_action(ActionEvents::ONGOING);
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionB>(&time, &mut actions, ActionState::None);
        set_action::<ActionC>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );

        set_action::<ActionA>(&time, &mut actions, ActionState::None);
        set_action::<ActionB>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionC>(&time, &mut actions, ActionState::Ongoing);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    /// Simulates action update to the desired state.
    fn set_action<A: InputAction>(
        time: &Time<Virtual>,
        actions: &mut ActionsData,
        state: ActionState,
    ) {
        let mut world = World::new();
        let mut action = ActionData::new::<A>();
        action.update(&mut world.commands(), time, &[], state, true);
        actions.insert_action::<A>(action);
    }

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct ActionA;

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct ActionB;

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct ActionC;

    #[derive(Debug, InputAction)]
    #[input_action(dim = Bool)]
    struct ActionD;
}
