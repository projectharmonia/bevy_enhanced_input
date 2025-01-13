use std::fmt::Debug;

use bevy::prelude::*;

use crate::action_value::{ActionValue, ActionValueDim};

/// Marker for a gameplay-related action.
///
/// Needs to be bind inside
/// [`InputContext::context_instance`](super::InputContext::context_instance)
///
/// Each binded action will have [`ActionState`](super::context_instance::ActionState).
/// When it updates during [`ContextInstance`](super::context_instance::ContextInstance)
/// evaluation, [`events`](super::events) are triggered.
///
/// Use observers to react on them:
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn move_character(trigger: Trigger<Fired<Move>>, mut transforms: Query<&mut Transform>) {
///    let event = trigger.event();
///    let mut transform = transforms.get_mut(trigger.entity()).unwrap();
///
///    // Since `Move` has `output = Vec2`, the value is `Vec2`.
///    // The value of the Z axis will be zero.
///    transform.translation += event.value.extend(0.0);
/// }
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec2)]
/// struct Move;
/// ```
///
/// You can also obtain the state directly from [`ActionData`](super::context_instance::ActionData),
/// see [`ContextInstances::context`](super::ContextInstances::context).
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
pub trait InputAction: Debug + Send + Sync + 'static {
    /// What type of value this action will output.
    ///
    /// - Use [`bool`] for button-like actions (e.g., `Jump`).
    /// - Use [`f32`] for single-axis actions (e.g., `Zoom`).
    /// - For multi-axis actions, like `Move`, use [`Vec2`] or [`Vec3`].
    ///
    /// The type here will determine the type of the `value` field on events
    /// e.g. [`Fired::value`](super::events::Fired::value),
    /// [`Canceled::value`](super::events::Canceled::value).
    type Output: ActionOutput;

    /// Specifies whether this action should swallow any [`Input`](crate::input::Input)s
    /// bound to it or allow them to pass through to affect other actions.
    ///
    /// Inputs are consumed only if the action state is not equal to
    /// [`ActionState::None`](super::context_instance::ActionState::None).
    /// For details, see [`ContextInstance`](super::context_instance::ContextInstance).
    ///
    /// Consuming is global and affect actions in all contexts.
    const CONSUME_INPUT: bool = true;

    /// Associated accumulation behavior.
    const ACCUMULATION: Accumulation = Accumulation::Cumulative;
}

/// Marks a type which can be used as [`InputAction::Output`].
pub trait ActionOutput: Send + Sync + Debug + Clone + Copy {
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
/// same most significant [`ActionState`](super::context_instance::ActionState)
/// (excluding [`ActionState::None`](super::context_instance::ActionState::None)).
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
