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

    /// List of input actions that need to be completed (according to [`ComboStep::events`]) to activate this action.
    ///
    /// Input actions must be triggered in order (starting at index 0) to count towards the triggering of the combo.
    pub steps: Vec<ComboStep>,
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

    fn cancel(&mut self) {
        self.step_index = 0;
        self.timer.reset();
    }

    fn cancelled(&self, actions_data: &ActionsData) -> bool {
        let current_step = &self.steps[self.step_index];
        match &current_step.cancel_actions {
            CancelActions::List(cancel_actions) => {
                for &(type_id, events) in cancel_actions {
                    if type_id == current_step.type_id {
                        continue;
                    }
                    let Some(action) = actions_data.get(&type_id) else {
                        warn_once!("cancel action is missing in context");
                        continue;
                    };

                    if action.events().intersects(events) {
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
            }
            CancelActions::All { events, exceptions } => {
                for (&type_id, action) in actions_data.iter() {
                    if type_id == current_step.type_id {
                        continue;
                    }
                    if let Some(allowed_events) = exceptions.get(&type_id) {
                        if allowed_events.contains(action.events()) {
                            continue;
                        }
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
        ConditionKind::Implicit
    }
}

/// Action and events that progress [`Combo`].
#[derive(Debug, Clone)]
pub struct ComboStep {
    /// Associated action.
    type_id: TypeId,

    /// Events for the action to complete this step.
    ///
    /// By default set to [`ActionEvents::FIRED`].
    pub events: ActionEvents,

    /// Time to trigger [`Self::events`] before the combo is cancelled.
    ///
    /// Starts once the previous step in the combo is completed.
    /// This can be safely ignored for the first action in the combo.
    ///
    /// By default set to [`f32::MAX`] which means infinity time.
    pub trigger_time: f32,

    /// Actions that may cancel the combo.
    pub cancel_actions: CancelActions,
}

impl ComboStep {
    /// Creates a new step for a combo.
    ///
    /// See also [`Combo::with_step`].
    pub fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            events: ActionEvents::FIRED,
            trigger_time: f32::MAX,
            cancel_actions: Default::default(),
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

    /// Cancel the combo if any other action in the same context (excluding the current step action) triggers any of the events.
    ///
    /// If [`CancelActions`] were set to [`CancelActions::List`], it will be replaced with [`CancelActions::All`].
    /// Use [`Self::allow`] and [`Self::allow_with`] to add exceptions.
    pub fn deny_any(mut self, all_events: ActionEvents) -> Self {
        match &mut self.cancel_actions {
            CancelActions::All { events, .. } => *events = all_events,
            CancelActions::List(_) => {
                self.cancel_actions = CancelActions::All {
                    events: all_events,
                    exceptions: Default::default(),
                }
            }
        }

        self
    }

    /// Like [`Self::allow_with`], but uses [`ActionEvents::all`] for the events.
    pub fn allow<A: InputAction>(self) -> Self {
        self.allow_with::<A>(ActionEvents::all())
    }

    /// Allow action `A` in the same context to trigger any of the given events without cancelling the combo.
    ///
    /// If [`CancelActions`] were set to [`CancelActions::List`], it will be replaced with [`CancelActions::All`].
    /// See also [`Self::allow`].
    pub fn allow_with<A: InputAction>(mut self, action_events: ActionEvents) -> Self {
        let type_id = TypeId::of::<A>();
        match &mut self.cancel_actions {
            CancelActions::All { exceptions, .. } => {
                exceptions.insert(type_id, action_events);
            }
            CancelActions::List(_) => {
                self.cancel_actions = CancelActions::All {
                    events: ActionEvents::all(),
                    exceptions: [(type_id, action_events)].into(),
                }
            }
        }

        self
    }

    /// Like [`Self::deny_with`], but uses [`ActionEvents::all`] for events.
    ///
    /// If [`CancelActions`] were set to [`CancelActions::All`], it will be replaced with [`CancelActions::List`].
    pub fn deny<A: InputAction>(self) -> Self {
        self.deny_with::<A>(ActionEvents::all())
    }

    /// Cancel the combo if action `A` in the same context triggers any of the events.
    ///
    /// See also [`Self::deny`].
    pub fn deny_with<A: InputAction>(mut self, events: ActionEvents) -> Self {
        let type_id = TypeId::of::<A>();
        match &mut self.cancel_actions {
            CancelActions::All { .. } => {
                self.cancel_actions = CancelActions::List(vec![(type_id, events)])
            }
            CancelActions::List(cancel_actions) => cancel_actions.push((type_id, events)),
        }

        self
    }
}

impl<A: InputAction> From<A> for ComboStep {
    fn from(_value: A) -> Self {
        ComboStep::new::<A>()
    }
}

/// Actions and events that cancel [`Combo`].
///
/// By default set to [`Self::All`] with [`ActionEvents::all`] and no exceptions.
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
            events: ActionEvents::all(),
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
    fn missing_first_step() {
        let mut condition = Combo::default().with_step(ActionA);
        let time = Time::default();
        let actions = ActionsData::default();

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn missing_next_step() {
        let mut condition = Combo::default().with_step(ActionA).with_step(ActionB);
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
        let mut condition = Combo::default()
            .with_step(ActionA)
            .with_step(ComboStep::new::<ActionB>().with_trigger_time(0.5));
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
        let mut condition = Combo::default().with_step(ActionA);
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
        let mut condition = Combo::default().with_step(ActionA).with_step(ActionB);
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
            .with_step(ActionA)
            .with_step(ComboStep::new::<ActionB>().deny::<ActionD>()) // Out of order is relevant only for `CancelActions::List`.
            .with_step(ActionC);
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
        let mut condition =
            Combo::default().with_step(ComboStep::new::<ActionA>().deny::<ActionA>());
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
    }

    #[test]
    fn missing_deny_action() {
        let mut condition =
            Combo::default().with_step(ComboStep::new::<ActionA>().deny::<ActionB>());
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
    }

    #[test]
    fn deny() {
        let mut condition = Combo::default()
            .with_step(ActionA)
            .with_step(ComboStep::new::<ActionB>().deny::<ActionC>());
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
    fn deny_any() {
        let mut condition = Combo::default()
            .with_step(ActionA)
            .with_step(ComboStep::new::<ActionB>().deny_any(ActionEvents::ONGOING));
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

    #[test]
    fn allow() {
        let mut condition = Combo::default()
            .with_step(ActionA)
            .with_step(ComboStep::new::<ActionB>().allow::<ActionC>());
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
            ActionState::Fired
        );
    }

    #[test]
    fn allow_and_deny() {
        let mut condition = Combo::default()
            .with_step(
                ComboStep::new::<ActionA>()
                    .deny::<ActionC>()
                    .allow::<ActionC>()
                    .allow::<ActionD>(),
            )
            .with_step(
                ComboStep::new::<ActionB>()
                    .allow::<ActionC>()
                    .allow::<ActionD>()
                    .deny::<ActionC>(),
            );
        let time = Time::default();
        let mut actions = ActionsData::default();
        set_action::<ActionA>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionB>(&time, &mut actions, ActionState::None);
        set_action::<ActionC>(&time, &mut actions, ActionState::Fired);
        set_action::<ActionD>(&time, &mut actions, ActionState::Fired);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );

        set_action::<ActionA>(&time, &mut actions, ActionState::None);
        set_action::<ActionB>(&time, &mut actions, ActionState::Fired);

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
