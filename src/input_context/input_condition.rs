pub mod primitives;

use std::{any, fmt::Debug, marker::PhantomData};

use bevy::prelude::*;

use super::input_action::{ActionState, ActionsData, InputAction};
use crate::action_value::ActionValue;
use primitives::{Actuation, HeldTimer};

pub trait InputCondition: Sync + Send + Debug + 'static {
    fn evaluate(
        &mut self,
        world: &World,
        actions_data: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState;

    fn kind(&self) -> ConditionKind {
        ConditionKind::Explicit
    }
}

/// Determines how a condition it contributes to [`ActionState`].
pub enum ConditionKind {
    Explicit,
    Implicit,
}

/// Returns [`ActionState::Fired`] when the input exceeds the actuation threshold.
#[derive(Default, Debug)]
pub struct Down {
    pub actuation: Actuation,
}

impl Down {
    pub fn new(actuation: Actuation) -> Self {
        Self { actuation }
    }
}

impl InputCondition for Down {
    fn evaluate(
        &mut self,
        _world: &World,
        _actions_data: &ActionsData,
        _delta: f32,
        value: ActionValue,
    ) -> ActionState {
        if self.actuation.is_actuated(value) {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

/// Like [`Down`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Default, Debug)]
pub struct Pressed {
    pub actuation: Actuation,
    actuated: bool,
}

impl Pressed {
    pub fn new(actuation: Actuation) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl InputCondition for Pressed {
    fn evaluate(
        &mut self,
        _world: &World,
        _actions_data: &ActionsData,
        _delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let previosly_actuated = self.actuated;
        self.actuated = self.actuation.is_actuated(value);

        if self.actuated && !previosly_actuated {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

/// Returns [`ActionState::Ongoing`]` when the input exceeds the actuation threshold and
/// [`ActionState::Fired`] once when the input drops back below the actuation threshold.
#[derive(Default, Debug)]
pub struct Released {
    pub actuation: Actuation,
    actuated: bool,
}

impl Released {
    pub fn new(actuation: Actuation) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl InputCondition for Released {
    fn evaluate(
        &mut self,
        _world: &World,
        _actions_data: &ActionsData,
        _delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let previosly_actuated = self.actuated;
        self.actuated = self.actuation.is_actuated(value);

        if self.actuated {
            // Ongoing on hold.
            ActionState::Ongoing
        } else if previosly_actuated {
            // Fired on release.
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

/// Returns [`ActionState::Ongoing`] when the input becomes actuated and
/// [`ActionState::Fired`] when input remained actuated for [`Self::hold_time`] seconds.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than [`Self::hold_time`] seconds.
/// May optionally fire once, or repeatedly fire.
#[derive(Debug)]
pub struct Hold {
    // How long does the input have to be held to cause trigger.
    pub hold_time: f32,

    // Should this trigger fire only once, or fire every frame once the hold time threshold is met?
    pub one_shot: bool,

    pub actuation: Actuation,

    held_timer: HeldTimer,

    fired: bool,
}

impl Hold {
    pub fn new(hold_time: f32) -> Self {
        Self {
            hold_time,
            one_shot: false,
            actuation: Default::default(),
            held_timer: Default::default(),
            fired: false,
        }
    }

    pub fn one_shot(mut self, one_shot: bool) -> Self {
        self.one_shot = one_shot;
        self
    }

    pub fn with_actuation(mut self, actuation: impl Into<Actuation>) -> Self {
        self.actuation = actuation.into();
        self
    }

    pub fn with_held_timer(mut self, held_timer: HeldTimer) -> Self {
        self.held_timer = held_timer;
        self
    }
}

impl InputCondition for Hold {
    fn evaluate(
        &mut self,
        world: &World,
        _actions_data: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let actuated = self.actuation.is_actuated(value);
        if actuated {
            self.held_timer.update(world, delta);
        } else {
            self.held_timer.reset();
        }

        let is_first_trigger = !self.fired;
        self.fired = self.held_timer.duration() >= self.hold_time;

        if self.fired {
            if is_first_trigger || !self.one_shot {
                ActionState::Fired
            } else {
                ActionState::None
            }
        } else if actuated {
            ActionState::Ongoing
        } else {
            ActionState::None
        }
    }
}

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released after having been actuated for [`Self::hold_time`] seconds.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than [`Self::hold_time`] seconds.
#[derive(Debug)]
pub struct HoldAndRelease {
    // How long does the input have to be held to cause trigger.
    pub hold_time: f32,

    pub actuation: Actuation,

    held_timer: HeldTimer,
}

impl HoldAndRelease {
    pub fn new(hold_time: f32) -> Self {
        Self {
            hold_time,
            actuation: Default::default(),
            held_timer: Default::default(),
        }
    }

    pub fn with_actuation(mut self, actuation: impl Into<Actuation>) -> Self {
        self.actuation = actuation.into();
        self
    }

    pub fn with_held_timer(mut self, held_timer: HeldTimer) -> Self {
        self.held_timer = held_timer;
        self
    }
}

impl InputCondition for HoldAndRelease {
    fn evaluate(
        &mut self,
        world: &World,
        _actions_data: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        // Evaluate the updated held duration prior to checking for actuation.
        // This stops us failing to trigger if the input is released on the
        // threshold frame due to held duration being 0.
        self.held_timer.update(world, delta);
        let held_duration = self.held_timer.duration();

        if self.actuation.is_actuated(value) {
            ActionState::Ongoing
        } else {
            self.held_timer.reset();
            // Trigger if we've passed the threshold and released.
            if held_duration > self.hold_time {
                ActionState::Fired
            } else {
                ActionState::None
            }
        }
    }
}

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released within the [`Self::release_time`] seconds.
///
/// Returns [`ActionState::None`] when the input is actuated more than [`Self::release_time`] seconds.
#[derive(Debug)]
pub struct Tap {
    pub release_time: f32,

    pub actuation: Actuation,

    held_timer: HeldTimer,
    actuated: bool,
}

impl Tap {
    pub fn new(release_time: f32) -> Self {
        Self {
            release_time,
            actuation: Default::default(),
            held_timer: Default::default(),
            actuated: false,
        }
    }

    pub fn with_actuation(mut self, actuation: impl Into<Actuation>) -> Self {
        self.actuation = actuation.into();
        self
    }

    pub fn with_held_timer(mut self, held_timer: HeldTimer) -> Self {
        self.held_timer = held_timer;
        self
    }
}

impl InputCondition for Tap {
    fn evaluate(
        &mut self,
        world: &World,
        _actions_data: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        let last_actuated = self.actuated;
        let last_held_duration = self.held_timer.duration();
        self.actuated = self.actuation.is_actuated(value);
        if self.actuated {
            self.held_timer.update(world, delta);
        } else {
            self.held_timer.reset();
        }

        if last_actuated && !self.actuated && last_held_duration <= self.release_time {
            // Only trigger if pressed then released quickly enough.
            ActionState::Fired
        } else if self.held_timer.duration() >= self.release_time {
            // Once we pass the threshold halt all triggering until released.
            ActionState::None
        } else if self.actuated {
            ActionState::Ongoing
        } else {
            ActionState::None
        }
    }
}

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// each [`Self::interval`] seconds.
///
/// Note: [`ActionEventKind::Completed`](super::input_action::ActionEventKind::Completed) only fires
/// when the repeat limit is reached or when input is released immediately after being triggered.
/// Otherwise, [`ActionEventKind::Canceled`](super::input_action::ActionEventKind::Canceled) is fired when input is released.
#[derive(Debug)]
pub struct Pulse {
    /// Time in seconds between each triggering while input is held.
    pub interval: f32,

    // Number of times the condition can be triggered (0 means no limit).
    pub trigger_limit: u32,

    /// Whether to trigger when the input first exceeds the actuation threshold or wait for the first interval.
    pub trigger_on_start: bool,

    pub actuation: Actuation,

    held_timer: HeldTimer,

    trigger_count: u32,
}

impl Pulse {
    pub fn new(interval: f32) -> Self {
        Self {
            interval,
            trigger_limit: 0,
            trigger_on_start: true,
            trigger_count: 0,
            actuation: Default::default(),
            held_timer: Default::default(),
        }
    }

    pub fn with_trigger_limit(mut self, trigger_limit: u32) -> Self {
        self.trigger_limit = trigger_limit;
        self
    }

    pub fn trigger_on_start(mut self, trigger_on_start: bool) -> Self {
        self.trigger_on_start = trigger_on_start;
        self
    }

    pub fn with_actuation(mut self, actuation: impl Into<Actuation>) -> Self {
        self.actuation = actuation.into();
        self
    }

    pub fn with_held_timer(mut self, held_timer: HeldTimer) -> Self {
        self.held_timer = held_timer;
        self
    }
}

impl InputCondition for Pulse {
    fn evaluate(
        &mut self,
        world: &World,
        _actions_data: &ActionsData,
        delta: f32,
        value: ActionValue,
    ) -> ActionState {
        if self.actuation.is_actuated(value) {
            self.held_timer.update(world, delta);

            if self.trigger_limit == 0 || self.trigger_count < self.trigger_limit {
                let trigger_count = if self.trigger_on_start {
                    self.trigger_count
                } else {
                    self.trigger_count + 1
                };

                // If the repeat count limit has not been reached.
                if self.held_timer.duration() > self.interval * trigger_count as f32 {
                    // Trigger when held duration exceeds the interval threshold.
                    self.trigger_count += 1;
                    ActionState::Fired
                } else {
                    ActionState::Ongoing
                }
            } else {
                ActionState::None
            }
        } else {
            self.held_timer.reset();

            self.trigger_count = 0;
            ActionState::None
        }
    }
}

/// Requires action `A` to be triggered within the same context.
///
/// Inherits [`ActionState`] from the specified action.
#[derive(Debug)]
pub struct Chord<A: InputAction> {
    pub marker: PhantomData<A>,
}

impl<A: InputAction> Default for Chord<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<A: InputAction> InputCondition for Chord<A> {
    fn evaluate(
        &mut self,
        _world: &World,
        actions_data: &ActionsData,
        _delta: f32,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(data) = actions_data.get_action::<A>() {
            // Inherit state from the chorded action.
            data.state()
        } else {
            warn_once!(
                "action `{}` is not present in context",
                any::type_name::<A>()
            );
            ActionState::None
        }
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}

/// Requires another action to not be triggered within the same context.
///
/// Could be used for chords to avoid triggering required actions.
#[derive(Debug)]
pub struct BlockedBy<A: InputAction> {
    pub marker: PhantomData<A>,
}

impl<A: InputAction> Default for BlockedBy<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<A: InputAction> InputCondition for BlockedBy<A> {
    fn evaluate(
        &mut self,
        _world: &World,
        actions_data: &ActionsData,
        _delta: f32,
        _value: ActionValue,
    ) -> ActionState {
        if let Some(data) = actions_data.get_action::<A>() {
            if data.state() == ActionState::Fired {
                return ActionState::None;
            }
        } else {
            warn_once!(
                "action `{}` is not present in context",
                any::type_name::<A>()
            );
        }

        ActionState::Fired
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}
