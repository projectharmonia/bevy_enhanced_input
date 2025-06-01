use core::fmt::Debug;

use bevy::prelude::*;

use crate::prelude::*;

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
