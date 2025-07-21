pub mod events;
pub mod fns;
pub mod relationship;
pub mod value;

use core::{any, fmt::Debug, time::Duration};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{context::input_reader::PendingInputs, prelude::*};
use fns::ActionFns;

/// Resets action data and triggers corresponding events on removal.
pub(crate) fn remove_action(
    trigger: Trigger<OnRemove, ActionValue>,
    mut commands: Commands,
    mut pending: ResMut<PendingInputs>,
    mut actions: Query<(
        Option<&Bindings>,
        &ActionSettings,
        &ActionFns,
        &mut ActionValue,
        &mut ActionState,
        &mut ActionEvents,
        &mut ActionTime,
    )>,
    bindings: Query<&Binding>,
) {
    let (action_bindings, settings, fns, mut value, mut state, mut events, mut time) =
        actions.get_mut(trigger.target()).unwrap();

    *time = Default::default();
    events.set_if_neq(ActionEvents::new(*state, ActionState::None));
    state.set_if_neq(Default::default());
    value.set_if_neq(ActionValue::zero(value.dim()));

    fns.trigger(
        &mut commands,
        trigger.target(),
        *state,
        *events,
        *value,
        *time,
    );

    if let Some(action_bindings) = action_bindings
        && settings.require_reset
    {
        pending.extend(bindings.iter_many(action_bindings).copied());
    }
}

/// Component that represents a user action.
///
/// Entities with this component needs to be spawned with [`ActionOf<C>`]
/// relationship in order to be evaluated.
///
/// Holds value defined by [`InputAction::Output`].
///
/// See the required components for other data associated with the action
/// that can be accessed without static typing.
#[derive(Component, Deref, DerefMut)]
#[require(
    Name::new(any::type_name::<A>()),
    ActionFns::new::<A>(),
    ActionValue::zero(A::Output::DIM),
    ActionSettings,
    ActionState,
    ActionEvents,
    ActionTime,
)]
pub struct Action<A: InputAction>(A::Output);

impl<A: InputAction> Clone for Action<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Action<A> {}

impl<A: InputAction> Action<A> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

/// Marker for a gameplay-related action.
///
/// Used together with [`Action<C>`] and [`events`] to statically define the type.
///
/// To implement the trait you can use the [`InputAction`](bevy_enhanced_input_macros::InputAction)
/// derive to reduce boilerplate:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(InputAction)]
/// #[action_output(Vec2)]
/// struct Move;
/// ```
pub trait InputAction: 'static {
    /// What type of value this action will output.
    ///
    /// - Use [`bool`] for button-like actions (e.g., `Jump`).
    /// - Use [`f32`] for single-axis actions (e.g., `Zoom`).
    /// - For multi-axis actions, like `Move`, use [`Vec2`] or [`Vec3`].
    type Output: ActionOutput;
}

/// Marks a type which can be used as [`InputAction::Output`].
pub trait ActionOutput: Into<ActionValue> + Default + Send + Sync + Debug + Clone + Copy {
    /// Dimension of this output.
    const DIM: ActionValueDim;

    /// Converts the value into the action output type.
    ///
    /// # Panics
    ///
    /// Panics if the value represents a different type.
    fn unwrap_value(value: ActionValue) -> Self;
}

impl ActionOutput for bool {
    const DIM: ActionValueDim = ActionValueDim::Bool;

    fn unwrap_value(value: ActionValue) -> Self {
        let ActionValue::Bool(value) = value else {
            panic!("output value should be bool");
        };
        value
    }
}

impl ActionOutput for f32 {
    const DIM: ActionValueDim = ActionValueDim::Axis1D;

    fn unwrap_value(value: ActionValue) -> Self {
        let ActionValue::Axis1D(value) = value else {
            panic!("output value should be axis 1D");
        };
        value
    }
}

impl ActionOutput for Vec2 {
    const DIM: ActionValueDim = ActionValueDim::Axis2D;

    fn unwrap_value(value: ActionValue) -> Self {
        let ActionValue::Axis2D(value) = value else {
            panic!("output value should be axis 2D");
        };
        value
    }
}

impl ActionOutput for Vec3 {
    const DIM: ActionValueDim = ActionValueDim::Axis3D;

    fn unwrap_value(value: ActionValue) -> Self {
        let ActionValue::Axis3D(value) = value else {
            panic!("output value should be axis 3D");
        };
        value
    }
}

/// Behavior configuration for [`Action<C>`].
#[derive(Component, Reflect, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ActionSettings {
    /// Accumulation behavior.
    ///
    /// By default set to [`Accumulation::default`].
    pub accumulation: Accumulation,

    /// Require inputs to be inactive before the first activation and continue to consume them
    /// even after context removal until inputs become inactive again.
    ///
    /// This way new instances won't react to currently held inputs until they are released.
    /// This prevents unintended behavior where switching or layering contexts using the same key
    /// could cause an immediate switch back, as buttons are rarely pressed for only a single frame.
    ///
    /// By default set to `false`.
    pub require_reset: bool,

    /// Specifies whether this action should swallow any [`Bindings`]
    /// bound to it or allow them to pass through to affect other actions.
    ///
    /// Inputs are consumed when the action state is not equal to
    /// [`ActionState::None`].
    ///
    /// Consuming is global and affect actions in all contexts.
    ///
    /// By default set to `true`.
    pub consume_input: bool,
}

impl Default for ActionSettings {
    fn default() -> Self {
        Self {
            accumulation: Default::default(),
            require_reset: false,
            consume_input: true,
        }
    }
}

/// Defines how [`ActionValue`] is calculated when multiple inputs are evaluated with the
/// same most significant [`ActionState`] (excluding [`ActionState::None`]).
///
/// Stored inside [`ActionSettings`].
#[derive(Reflect, Debug, Default, Serialize, Deserialize, Clone, Copy)]
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

/// State for [`Action<C>`].
///
/// Updated from [`Bindings`] and associated [`conditions`](crate::condition),
/// or overridden by [`ActionMock`] if present.
///
/// During evaluation, [`ActionEvents`] are derived from the previous and current state.
#[derive(
    Component,
    Reflect,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
)]
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

/// Timing information for [`Action<C>`].
#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
pub struct ActionTime {
    /// Time the action was in [`ActionState::Ongoing`] and [`ActionState::Fired`] states.
    pub elapsed_secs: f32,

    /// Time the action was in [`ActionState::Fired`] state.
    pub fired_secs: f32,
}

impl ActionTime {
    pub(crate) fn update(&mut self, delta_secs: f32, state: ActionState) {
        match state {
            ActionState::None => {
                self.elapsed_secs = 0.0;
                self.fired_secs = 0.0;
            }
            ActionState::Ongoing => {
                self.elapsed_secs += delta_secs;
                self.fired_secs = 0.0;
            }
            ActionState::Fired => {
                self.elapsed_secs += delta_secs;
                self.fired_secs += delta_secs;
            }
        }
    }
}

/// Mocks the state and value of [`Action<C>`] for a specified span.
///
/// While active, input evaluation, conditions, and modifiers are skipped. Instead,
/// the action reports the provided state and value. All state transition events
/// (e.g., [`Started<A>`], [`Fired<A>`]) will still be triggered as usual.
///
/// Once the span expires, [`Self::enabled`] is set to `false`, and the action resumes
/// evaluating real input. The component is not removed automatically, allowing you
/// to reuse it for future mocking.
///
/// Mocking does not take effect immediately - it is applied during the next context evaluation.
/// For more details, see the [evaluation](../index.html#evaluation) section in the quick start guide.
///
/// # Examples
///
/// Spawn and move up for 2 seconds:
///
/// ```
/// # use core::time::Duration;
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn((
///     OnFoot,
///     actions!(OnFoot[
///         (
///             Action::<Move>::new(),
///             ActionMock::new(ActionState::Fired, Vec2::Y, Duration::from_secs(2)),
///             Bindings::spawn(Cardinal::wasd_keys()), // Bindings will be ignored while mocked.
///         ),
///     ]),
/// ));
/// # #[derive(Component)]
/// # struct OnFoot;
/// # #[derive(InputAction)]
/// # #[action_output(Vec2)]
/// # struct Move;
/// ```
///
/// Mock previously spawned jump action for the next frame:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// fn mock_jump(mut commands: Commands, jump: Single<Entity, With<Action<Jump>>>) {
///     commands.entity(*jump).insert(ActionMock::once(ActionState::Fired, true));
/// }
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Jump;
#[derive(Component, Reflect, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ActionMock {
    pub state: ActionState,
    pub value: ActionValue,
    pub span: MockSpan,
    pub enabled: bool,
}

impl ActionMock {
    /// Creates a new instance that will mock state and value only for a single context evaluation.
    pub fn once(state: ActionState, value: impl Into<ActionValue>) -> Self {
        Self::new(state, value, MockSpan::Updates(1))
    }

    /// Creates a new instance that will mock state and value for the given span.
    pub fn new(
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> Self {
        Self {
            state,
            value: value.into(),
            span: span.into(),
            enabled: true,
        }
    }
}

/// Specifies how long [`ActionMock`] should remain active.
#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MockSpan {
    /// Active for a fixed number of context evaluations.
    Updates(u32),
    /// Active for a real-time [`Duration`].
    Duration(Duration),
    /// Remains active until [`ActionMock::enabled`] is manually set to `false`,
    /// or the [`ActionMock`] component is removed from the action entity.
    Manual,
}

impl From<Duration> for MockSpan {
    fn from(value: Duration) -> Self {
        Self::Duration(value)
    }
}
