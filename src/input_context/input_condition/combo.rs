use std::any::TypeId;

use bevy::{prelude::*, utils::HashMap};

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

    /// List of actions that need to be completed (according to [`ComboStep::events`]) to activate this action.
    ///
    /// Input actions must be triggered in order (starting at index 0) to count towards the triggering of the combo.
    pub steps: Vec<ComboStep>,

    /// Actions from the current context that will cancel the combo if they trigger [`CancelAction::events`].
    ///
    /// If a cancel action matches the action from the current step, it will be ignored.
    pub cancel_actions: Vec<CancelAction>,
}

impl Combo {
    /// Adds an action step to the combo.
    ///
    /// If you don't need to configure the step, you can just pass the action directly:
    ///
    /// ```
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut combo = Combo::default();
    /// combo.with_step(Jump);
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(dim = Bool)]
    /// # struct Jump;
    /// ```
    pub fn with_step(mut self, step: impl Into<ComboStep>) -> Self {
        self.steps.push(step.into());
        self
    }

    /// Adds an action that cancels the combo.
    ///
    /// If you don't need to configure the events, you can just pass the action directly:
    ///
    /// ```
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut combo = Combo::default();
    /// combo.with_cancel(Jump);
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(dim = Bool)]
    /// # struct Jump;
    /// ```
    pub fn with_cancel(mut self, cancel_action: impl Into<CancelAction>) -> Self {
        self.cancel_actions.push(cancel_action.into());
        self
    }

    fn cancel(&mut self) {
        self.step_index = 0;
        self.timer.reset();
    }

    fn cancelled(&self, actions_data: &ActionsData) -> bool {
        let current_step = &self.steps[self.step_index];
        for cancel_action in &self.cancel_actions {
            if cancel_action.type_id == current_step.type_id {
                continue;
            }
            let Some(action) = actions_data.get(&cancel_action.type_id) else {
                warn_once!("cancel action is missing in context");
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
                warn_once!("step action is missing in context");
                continue;
            };

            if action.events().intersects(step.events) {
                return true;
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
            warn_once!("combo has no combo steps");
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

        let current_step = &self.steps[self.step_index];
        let Some(current_action) = actions_data.get(&current_step.type_id) else {
            warn_once!("current step action is missing in context");
            self.cancel();
            return ActionState::None;
        };

        if current_action.events().contains(current_step.events) {
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
        ConditionKind::Implicit
    }
}

/// An action and events that progress [`Combo`].
#[derive(Debug, Clone)]
pub struct ComboStep {
    /// Associated action.
    type_id: TypeId,

    /// Events for the action to complete this step.
    ///
    /// By default set to [`ActionEvents::COMPLETED`].
    pub events: ActionEvents,

    /// Time to trigger [`Self::events`] before the combo is cancelled.
    ///
    /// Starts once the previous step in the combo is completed.
    /// This can be safely ignored for the first action in the combo.
    ///
    /// By default set to [`f32::MAX`] which means infinity time.
    pub trigger_time: f32,
}

impl ComboStep {
    /// Creates a default step for action `A`.
    ///
    /// See also [`Combo::with_step`].
    pub fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            events: ActionEvents::COMPLETED,
            trigger_time: f32::MAX,
        }
    }

    /// Replaces [`Self::trigger_time`] time with the given value.
    pub fn with_trigger_time(mut self, trigger_time: f32) -> Self {
        self.trigger_time = trigger_time;
        self
    }

    /// Replaces [`Self::events`] with the given value.
    pub fn with_events(mut self, events: ActionEvents) -> Self {
        self.events = events;
        self
    }
}

impl<A: InputAction> From<A> for ComboStep {
    fn from(_value: A) -> Self {
        ComboStep::new::<A>()
    }
}

/// An action and events that cancel a [`Combo`].
#[derive(Debug, Clone)]
pub struct CancelAction {
    /// Associated action.
    type_id: TypeId,

    /// Events for the action that will cancel a combo.
    ///
    /// By default set to [`ActionEvents::ONGOING`] and [`ActionEvents::FIRED`]
    pub events: ActionEvents,
}

impl CancelAction {
    /// Creates a default cancel action.
    ///
    /// See also [`Combo::with_cancel`].
    pub fn new<A: InputAction>() -> Self {
        Self::with::<A>(ActionEvents::ONGOING | ActionEvents::FIRED)
    }

    /// Creates a cancel action with the given events.
    ///
    /// See also [`Combo::with_cancel`].
    pub fn with<A: InputAction>(events: ActionEvents) -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            events,
        }
    }
}

impl<A: InputAction> From<A> for CancelAction {
    fn from(_value: A) -> Self {
        CancelAction::new::<A>()
    }
}

/// Actions and events that cancel [`Combo`].
///
/// By default set to [`Self::All`] with [`ActionEvents::ONGOING`] and [`ActionEvents::FIRED`] and no exceptions.
#[derive(Debug, Clone)]
pub enum CancelActions {
    /// Cancel the combo if any action (excluding the action for the current step and exception) trigger the given events.
    All {
        /// Events that cancel the action.
        events: ActionEvents,
        /// Actions and associated events they allowed to trigger without cancelling the combo.
        exceptions: HashMap<TypeId, ActionEvents>,
    },
    /// Cancel the combo if any listed action (excluding the action for the current step) triggers associated events.
    ///
    /// With this variant actions from other steps will also be checked for not triggering their events.
    List(Vec<(TypeId, ActionEvents)>),
}

impl Default for CancelActions {
    fn default() -> Self {
        Self::All {
            events: ActionEvents::ONGOING | ActionEvents::FIRED,
            exceptions: Default::default(),
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
    fn missing_step() {
        let mut condition = Combo::default().with_step(ActionA).with_step(ActionB);
        let time = Time::default();
        let actions = ActionsData::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn timeout() {
        let mut condition = Combo::default()
            .with_step(ActionA)
            .with_step(ComboStep::new::<ActionB>().with_trigger_time(0.5));
        let mut time = Time::default();
        time.advance_by(Duration::from_secs(1));
        let mut actions = ActionsData::default();
        transition::<ActionA>(&time, &mut actions, ActionState::Fired);
        transition::<ActionA>(&time, &mut actions, ActionState::None);
        transition::<ActionB>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing,
            "first step shouldn't be affected by time"
        );
        assert_eq!(condition.step_index, 1);

        time.advance_by(Duration::from_secs(1));
        transition::<ActionA>(&time, &mut actions, ActionState::None); // Clear `Completed` event.
        transition::<ActionB>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn first_step_ongoing() {
        let mut condition = Combo::default().with_step(ActionA);
        let time = Time::default();
        let mut actions = ActionsData::default();
        transition::<ActionA>(&time, &mut actions, ActionState::Ongoing);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
    }

    #[test]
    fn steps() {
        let mut condition = Combo::default().with_step(ActionA).with_step(ActionB);
        let time = Time::default();
        let mut actions = ActionsData::default();
        transition::<ActionA>(&time, &mut actions, ActionState::Fired);
        transition::<ActionA>(&time, &mut actions, ActionState::None);
        transition::<ActionB>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 1);

        transition::<ActionA>(&time, &mut actions, ActionState::None);
        transition::<ActionB>(&time, &mut actions, ActionState::Fired);
        transition::<ActionB>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(condition.step_index, 0);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn out_of_order() {
        let mut condition = Combo::default()
            .with_step(ActionA)
            .with_step(ActionB)
            .with_step(ActionC);
        let time = Time::default();
        let mut actions = ActionsData::default();
        transition::<ActionA>(&time, &mut actions, ActionState::None);
        transition::<ActionB>(&time, &mut actions, ActionState::Fired);
        transition::<ActionC>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);

        transition::<ActionB>(&time, &mut actions, ActionState::None);
        transition::<ActionB>(&time, &mut actions, ActionState::None);
        transition::<ActionA>(&time, &mut actions, ActionState::Fired);
        transition::<ActionA>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 1);

        transition::<ActionA>(&time, &mut actions, ActionState::None);
        transition::<ActionC>(&time, &mut actions, ActionState::Fired);
        transition::<ActionC>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn ignore_same_cancel_action() {
        let mut condition = Combo::default().with_step(ActionA).with_cancel(ActionA);
        let time = Time::default();
        let mut actions = ActionsData::default();
        transition::<ActionA>(&time, &mut actions, ActionState::Fired);
        transition::<ActionA>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn missing_cancel_action() {
        let mut condition = Combo::default().with_step(ActionA).with_cancel(ActionB);
        let time = Time::default();
        let mut actions = ActionsData::default();
        transition::<ActionA>(&time, &mut actions, ActionState::Fired);
        transition::<ActionA>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn cancel() {
        let mut condition = Combo::default()
            .with_step(ActionA)
            .with_step(ActionB)
            .with_cancel(ActionC);
        let time = Time::default();
        let mut actions = ActionsData::default();
        transition::<ActionA>(&time, &mut actions, ActionState::Fired);
        transition::<ActionA>(&time, &mut actions, ActionState::None);
        transition::<ActionB>(&time, &mut actions, ActionState::None);
        transition::<ActionC>(&time, &mut actions, ActionState::None);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 1);

        transition::<ActionA>(&time, &mut actions, ActionState::None);
        transition::<ActionB>(&time, &mut actions, ActionState::Fired);
        transition::<ActionB>(&time, &mut actions, ActionState::None);
        transition::<ActionC>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    /// Simulates action update to the desired state.
    fn transition<A: InputAction>(
        time: &Time<Virtual>,
        actions: &mut ActionsData,
        state: ActionState,
    ) {
        let action = actions
            .action_entry::<A>()
            .or_insert_with(ActionData::new::<A>);
        let mut world = World::new();
        action.update(&mut world.commands(), time, &[], state, true);
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
}
