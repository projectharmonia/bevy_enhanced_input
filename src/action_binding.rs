use alloc::{boxed::Box, vec::Vec};
use core::{
    any::{self, TypeId},
    cmp::Ordering,
    time::Duration,
};

use bevy::prelude::*;
use log::{debug, trace};

use crate::{
    action_map::ActionMap, input_action::ActionOutput, input_condition::IntoConditions,
    input_modifier::IntoModifiers, input_reader::InputReader, prelude::*,
    trigger_tracker::TriggerTracker,
};

/// Bindings associated with an [`InputAction`] marker.
///
/// Stored inside [`Actions`].
///
/// Bindings are stored separately from [`ActionMap`] to allow reading other actions' data during evaluation.
///
/// Action bindings evaluation follows these steps:
///
/// 1. Iterate over each [`ActionValue`] from the associated [`Input`]s:
///    1.1. Apply input-level [`InputModifier`]s.
///    1.2. Evaluate input-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
/// 2. Select all [`ActionValue`]s with the most significant [`ActionState`] and combine based on [`InputAction::ACCUMULATION`].
///    Combined value will be converted into [`InputAction::Output`] using [`ActionValue::convert`].
/// 3. Apply action level [`InputModifier`]s.
/// 4. Evaluate action level [`InputCondition`]s, combining their results according to [`InputCondition::kind`].
/// 5. Set the final [`ActionState`] based on the results.
///    Final value will be converted into [`InputAction::Output`] using [`ActionValue::convert`].
pub struct ActionBinding {
    type_id: TypeId,
    action_name: &'static str,
    dim: ActionValueDim,
    consume_input: bool,
    accumulation: Accumulation,
    require_reset: bool,

    mock: Option<ActionMock>,
    modifiers: Vec<Box<dyn InputModifier>>,
    conditions: Vec<Box<dyn InputCondition>>,
    inputs: Vec<InputBinding>,

    /// Consumed inputs during state evaluation.
    consume_buffer: Vec<Input>,
}

impl ActionBinding {
    #[must_use]
    pub(crate) fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            action_name: any::type_name::<A>(),
            dim: A::Output::DIM,
            consume_input: A::CONSUME_INPUT,
            accumulation: A::ACCUMULATION,
            require_reset: A::REQUIRE_RESET,
            mock: None,
            modifiers: Default::default(),
            conditions: Default::default(),
            inputs: Default::default(),
            consume_buffer: Default::default(),
        }
    }

    /// Returns associated input bindings.
    ///
    /// See also [`Self::to`].
    pub fn inputs(&self) -> &[InputBinding] {
        &self.inputs
    }

    /// Adds action-level modifiers.
    ///
    /// For input-level modifiers see [`BindingBuilder::with_modifiers`].
    ///
    /// # Examples
    ///
    /// Single modifier:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_modifiers(Scale::splat(2.0));
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Jump;
    /// ```
    ///
    /// Multiple modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_modifiers((Scale::splat(2.0), Negate::all()));
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Jump;
    /// ```
    pub fn with_modifiers(&mut self, modifiers: impl IntoModifiers) -> &mut Self {
        for modifier in modifiers.into_modifiers() {
            debug!("adding `{modifier:?}` to `{}`", self.action_name);
            self.modifiers.push(modifier);
        }

        self
    }

    /// Adds action-level conditions.
    ///
    /// For input-level conditions see [`BindingBuilder::with_conditions`].
    ///
    /// # Examples
    ///
    /// Single condition:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_conditions(Release::default());
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// Multiple conditions:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_conditions((Release::default(), Press::default()));
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    pub fn with_conditions(&mut self, conditions: impl IntoConditions) -> &mut Self {
        for condition in conditions.into_conditions() {
            debug!("adding `{condition:?}` to `{}`", self.action_name);
            self.conditions.push(condition);
        }

        self
    }

    /// Adds input mapping.
    ///
    /// Thanks to traits, this function can be called with multiple types:
    ///
    /// 1. Raw input types.
    /// 2. [`Input`] enum which wraps any supported raw input and can store keyboard modifiers.
    /// 3. [`InputBinding`] which wraps [`Input`] and can store input modifiers or conditions.
    /// 4. [`IntoBindings`] which wraps [`InputBinding`] and can store multiple [`InputBinding`]s.
    ///    Also implemented on tuples, so you can pass multiple inputs to a single call.
    ///
    /// All assigned inputs will be evaluated separately (equivalent to "any of").
    /// If you're looking for a chord, see the [`Chord`] condition.
    ///
    /// # Examples
    ///
    /// Raw input:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Jump>()
    ///     .to((KeyCode::Space, GamepadButton::South));
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// Raw input with keyboard modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Jump>().to(KeyCode::Space.with_mod_keys(ModKeys::CONTROL));
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// Raw input with input conditions or modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Jump>().to(KeyCode::Space.with_conditions(Release::default()));
    /// actions.bind::<Attack>().to(MouseButton::Left.with_modifiers(Scale::splat(10.0)));
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = f32)]
    /// # struct Attack;
    /// ```
    ///
    /// [`Input`] type directly:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Zoom>().to(Input::mouse_wheel());
    /// actions.bind::<Move>().to(Input::mouse_motion());
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Zoom;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = Vec2)]
    /// # struct Move;
    /// ```
    ///
    /// Convenience preset which consists of multiple inputs
    /// with predefined conditions and modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// actions.bind::<Move>().to(Cardinal::wasd_keys());
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = Vec2)]
    /// # struct Move;
    /// ```
    ///
    /// Multiple buttons from settings:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Player>::default();
    /// # let mut settings = KeyboardSettings::default();
    /// actions.bind::<Inspect>().to(&settings.inspect);
    ///
    /// # #[derive(Default)]
    /// struct KeyboardSettings {
    ///     inspect: Vec<KeyCode>,
    /// }
    /// # #[derive(InputContext)]
    /// # struct Player;
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = Vec2)]
    /// # struct Inspect;
    /// ```
    pub fn to(&mut self, bindings: impl IntoBindings) -> &mut Self {
        for binding in bindings.into_bindings() {
            debug!("adding `{binding:?}` to `{}`", self.action_name);
            self.inputs.push(binding);
        }
        self
    }

    /// Type-erased version for [`Actions::mock`].
    pub(crate) fn mock(&mut self, state: ActionState, value: ActionValue, span: MockSpan) {
        debug!(
            "mocking `{}` with `{state:?}` and `{value:?}` for `{span:?}`",
            self.action_name,
        );
        self.mock = Some(ActionMock { state, value, span });
    }

    pub(crate) fn clear_mock(&mut self) {
        debug!("clearing mock from `{}`", self.action_name);
        self.mock = None;
    }

    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        action_map: &mut ActionMap,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        let (state, value) = self
            .update_from_mock(time.delta())
            .unwrap_or_else(|| self.update_from_reader(reader, action_map, time));

        let action = action_map
            .get_mut(&self.type_id)
            .expect("actions and bindings should have matching type IDs");

        action.update(time, state, value);
        action.trigger_events(commands, entity);
    }

    pub(crate) fn update_from_reader(
        &mut self,
        reader: &mut InputReader,
        action_map: &mut ActionMap,
        time: &Time<Virtual>,
    ) -> (ActionState, ActionValue) {
        trace!("updating `{}` from input", self.action_name);

        let mut tracker = TriggerTracker::new(ActionValue::zero(self.dim));
        for binding in &mut self.inputs {
            let value = reader.value(binding.input);
            if self.require_reset && binding.first_activation {
                // Ignore until we read zero for this mapping.
                if value.as_bool() {
                    continue;
                } else {
                    binding.first_activation = false;
                }
            }

            let mut current_tracker = TriggerTracker::new(value);
            trace!("reading value `{value:?}`");
            current_tracker.apply_modifiers(action_map, time, &mut binding.modifiers);
            current_tracker.apply_conditions(action_map, time, &mut binding.conditions);

            let current_state = current_tracker.state();
            if current_state == ActionState::None {
                // Ignore non-active trackers to allow the action to fire even if all
                // input-level conditions return `ActionState::None`. This ensures that an
                // action-level condition or modifier can still trigger the action.
                continue;
            }

            match current_state.cmp(&tracker.state()) {
                Ordering::Less => (),
                Ordering::Equal => {
                    tracker.combine(current_tracker, self.accumulation);
                    if self.consume_input {
                        self.consume_buffer.push(binding.input);
                    }
                }
                Ordering::Greater => {
                    tracker.overwrite(current_tracker);
                    if self.consume_input {
                        self.consume_buffer.clear();
                        self.consume_buffer.push(binding.input);
                    }
                }
            }
        }

        tracker.apply_modifiers(action_map, time, &mut self.modifiers);
        tracker.apply_conditions(action_map, time, &mut self.conditions);

        let state = tracker.state();
        let value = tracker.value().convert(self.dim);

        if self.consume_input {
            if state != ActionState::None {
                for &input in &self.consume_buffer {
                    reader.consume(input);
                }
            }
            self.consume_buffer.clear();
        }

        (state, value)
    }

    fn update_from_mock(&mut self, delta: Duration) -> Option<(ActionState, ActionValue)> {
        let Some(mock) = &mut self.mock else {
            return None;
        };
        trace!("updating `{}` from `{mock:?}`", self.action_name);

        let expired = match &mut mock.span {
            MockSpan::Updates(ticks) => {
                *ticks = ticks.saturating_sub(1);
                *ticks == 0
            }
            MockSpan::Duration(duration) => {
                *duration = duration.saturating_sub(delta);
                trace!("reducing mock duration by {delta:?}");
                duration.is_zero()
            }
            MockSpan::Manual => false,
        };

        let state = mock.state;
        let value = mock.value;
        if expired {
            self.clear_mock();
        }

        Some((state, value))
    }

    pub(crate) fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub(crate) fn dim(&self) -> ActionValueDim {
        self.dim
    }

    pub(crate) fn require_reset(&self) -> bool {
        self.require_reset
    }

    pub(crate) fn max_mod_keys(&self) -> usize {
        self.inputs()
            .iter()
            .map(|b| b.input.mod_keys_count())
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug)]
struct ActionMock {
    state: ActionState,
    value: ActionValue,
    span: MockSpan,
}

/// Specifies how long a mock input should remain active.
///
/// See also [`Actions::mock`].
#[derive(Clone, Copy, Debug)]
pub enum MockSpan {
    /// Mock for a fixed number of context evaluations.
    Updates(u32),
    /// Mock for a real-time duration.
    Duration(Duration),
    /// Mock until manually cleared.
    Manual,
}

impl From<Duration> for MockSpan {
    fn from(value: Duration) -> Self {
        Self::Duration(value)
    }
}
