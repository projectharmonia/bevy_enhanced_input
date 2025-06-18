use core::{any, fmt::Debug};

use bevy::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Data associated with an [`InputAction`] marker.
///
/// Stored inside [`Actions`].
#[derive(Clone, Copy)]
pub struct Action {
    state: ActionState,
    events: ActionEvents,
    value: ActionValue,
    elapsed_secs: f32,
    fired_secs: f32,
    trigger_events: fn(&Self, &mut Commands, Entity),
}

impl Action {
    /// Creates a new instance associated with action `A`.
    ///
    /// [`Self::trigger_events`] will trigger events for `A`.
    #[must_use]
    pub(crate) fn new<A: InputAction>() -> Self {
        Self {
            state: Default::default(),
            events: ActionEvents::empty(),
            value: ActionValue::zero(A::Output::DIM),
            elapsed_secs: 0.0,
            fired_secs: 0.0,
            trigger_events: Self::trigger_events_typed::<A>,
        }
    }

    /// Updates internal state.
    pub(crate) fn update(
        &mut self,
        time: &InputTime,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) {
        match self.state {
            ActionState::None => {
                self.elapsed_secs = 0.0;
                self.fired_secs = 0.0;
            }
            ActionState::Ongoing => {
                self.elapsed_secs += time.delta_secs();
                self.fired_secs = 0.0;
            }
            ActionState::Fired => {
                self.elapsed_secs += time.delta_secs();
                self.fired_secs += time.delta_secs();
            }
        }

        self.events = ActionEvents::new(self.state, state);
        self.state = state;
        self.value = value.into();
    }

    /// Triggers events resulting from a state transition after [`Self::update`].
    ///
    /// See also [`Self::new`] and [`ActionEvents`].
    pub(crate) fn trigger_events(&self, commands: &mut Commands, entity: Entity) {
        (self.trigger_events)(self, commands, entity);
    }

    /// A typed version of [`Self::trigger_events`].
    fn trigger_events_typed<A: InputAction>(&self, commands: &mut Commands, entity: Entity) {
        for (_, event) in self.events.iter_names() {
            match event {
                ActionEvents::STARTED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Started::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                        },
                    );
                }
                ActionEvents::ONGOING => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Ongoing::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::FIRED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Fired::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                            fired_secs: self.fired_secs,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::CANCELED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Canceled::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                ActionEvents::COMPLETED => {
                    trigger_and_log::<A, _>(
                        commands,
                        entity,
                        Completed::<A> {
                            value: A::Output::as_output(self.value),
                            state: self.state,
                            fired_secs: self.fired_secs,
                            elapsed_secs: self.elapsed_secs,
                        },
                    );
                }
                _ => unreachable!("iteration should yield only named flags"),
            }
        }
    }

    /// Returns the current state.
    pub fn state(&self) -> ActionState {
        self.state
    }

    /// Returns events triggered by a transition of [`Self::state`] since the last update.
    pub fn events(&self) -> ActionEvents {
        self.events
    }

    /// Returns the value since the last update.
    ///
    /// Unlike when reading values from triggers, this returns [`ActionValue`] since actions
    /// are stored in a type-erased format.
    pub fn value(&self) -> ActionValue {
        self.value
    }

    /// Time the action was in [`ActionState::Ongoing`] and [`ActionState::Fired`] states.
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed_secs
    }

    /// Time the action was in [`ActionState::Fired`] state.
    pub fn fired_secs(&self) -> f32 {
        self.fired_secs
    }
}

fn trigger_and_log<A, E: Event + Debug>(commands: &mut Commands, entity: Entity, event: E) {
    debug!(
        "triggering `{event:?}` for `{}` for `{entity}`",
        any::type_name::<A>()
    );
    commands.trigger_targets(event, entity);
}

/// State for [`Action`].
///
/// States are ordered by their significance.
///
/// See also [`ActionEvents`] and [`ActionBinding`].
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionState {
    /// Condition is not triggered.
    #[default]
    None,
    /// Condition has started triggering, but has not yet finished.
    ///
    /// For example, [`Hold`] condition requires its state to be
    /// maintained over several frames.
    Ongoing,
    /// The condition has been met.
    Fired,
}

/// Marker for a gameplay-related action.
///
/// Needs to be bound to actions using [`Actions::bind`](crate::actions::Actions::bind).
///
/// To implement the trait you can use the [`InputAction`](bevy_enhanced_input_macros::InputAction)
/// derive to reduce boilerplate:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec2)]
/// struct Move;
/// ```
///
/// Optionally you can pass `consume_input` and/or `accumulation`:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec2, accumulation = Cumulative, consume_input = false)]
/// struct Move;
/// ```
///
/// All parameters match corresponding data in the trait.
pub trait InputAction: Debug + Send + Sync + 'static {
    /// What type of value this action will output.
    ///
    /// - Use [`bool`] for button-like actions (e.g., `Jump`).
    /// - Use [`f32`] for single-axis actions (e.g., `Zoom`).
    /// - For multi-axis actions, like `Move`, use [`Vec2`] or [`Vec3`].
    ///
    /// This type will also be used for `value` field on events
    /// e.g. [`Fired::value`], [`Canceled::value`].
    type Output: ActionOutput;

    /// Specifies whether this action should swallow any [`Input`]s
    /// bound to it or allow them to pass through to affect other actions.
    ///
    /// Inputs are consumed when the action state is not equal to
    /// [`ActionState::None`]. For details, see [`Actions`].
    ///
    /// Consuming is global and affect actions in all contexts.
    const CONSUME_INPUT: bool = true;

    /// Associated accumulation behavior.
    const ACCUMULATION: Accumulation = Accumulation::Cumulative;

    /// Require inputs to be zero before the first activation and continue to consume them
    /// even after context removal until inputs become zero again.
    ///
    /// This way new instances won't react to currently held inputs until they are released.
    /// This prevents unintended behavior where switching or layering contexts using the same key
    /// could cause an immediate switch back, as buttons are rarely pressed for only a single frame.
    const REQUIRE_RESET: bool = false;
}

/// Marks a type which can be used as [`InputAction::Output`].
pub trait ActionOutput: Send + Sync + Debug + Clone + Copy + Into<ActionValue> {
    /// Dimension of this output.
    const DIM: ActionValueDim;

    /// Converts the value into the action output type.
    ///
    /// # Panics
    ///
    /// Panics if the value represents a different type.
    fn as_output(value: ActionValue) -> Self;
}

impl ActionOutput for bool {
    const DIM: ActionValueDim = ActionValueDim::Bool;

    fn as_output(value: ActionValue) -> Self {
        let ActionValue::Bool(value) = value else {
            unreachable!("output value should be bool");
        };
        value
    }
}

impl ActionOutput for f32 {
    const DIM: ActionValueDim = ActionValueDim::Axis1D;

    fn as_output(value: ActionValue) -> Self {
        let ActionValue::Axis1D(value) = value else {
            unreachable!("output value should be axis 1D");
        };
        value
    }
}

impl ActionOutput for Vec2 {
    const DIM: ActionValueDim = ActionValueDim::Axis2D;

    fn as_output(value: ActionValue) -> Self {
        let ActionValue::Axis2D(value) = value else {
            unreachable!("output value should be axis 2D");
        };
        value
    }
}

impl ActionOutput for Vec3 {
    const DIM: ActionValueDim = ActionValueDim::Axis3D;

    fn as_output(value: ActionValue) -> Self {
        let ActionValue::Axis3D(value) = value else {
            unreachable!("output value should be axis 3D");
        };
        value
    }
}

/// Defines how [`ActionValue`] is calculated when multiple inputs are evaluated with the
/// same most significant [`ActionState`] (excluding [`ActionState::None`]).
#[derive(Default, Clone, Copy, Debug)]
pub enum Accumulation {
    /// Cumulatively add the key values for each mapping.
    ///
    /// For example, given values of 0.5 and -0.3, the input action's value would be 0.2.
    ///
    /// Usually used for things like WASD movement, when you want pressing W and S to cancel each other out.
    #[default]
    Cumulative,
    /// Take the value from the mapping with the highest absolute value.
    ///
    /// For example, given values of 0.5 and -1.5, the input action's value would be -1.5.
    MaxAbs,
}
