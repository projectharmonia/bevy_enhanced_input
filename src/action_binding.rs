use alloc::{boxed::Box, vec::Vec};
use core::{
    any::{self, TypeId},
    cmp::Ordering,
};

use bevy::prelude::*;

use crate::{
    action_map::{ActionMap, ActionState},
    action_value::{ActionValue, ActionValueDim},
    input::Input,
    input_action::{Accumulation, ActionOutput, InputAction},
    input_binding::{InputBinding, IntoBindings},
    input_condition::{InputCondition, IntoConditions},
    input_modifier::{InputModifier, IntoModifiers},
    input_reader::InputReader,
    trigger_tracker::TriggerTracker,
};

/// Bindings of [`InputAction`] for [`Actions`](crate::actions::Actions).
///
/// These bindings are stored separately from [`ActionMap`] to allow a currently
/// evaluating action to access the state of other actions.
pub struct ActionBinding {
    type_id: TypeId,
    action_name: &'static str,
    dim: ActionValueDim,
    consume_input: bool,
    accumulation: Accumulation,
    require_reset: bool,

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
    /// For input-level modifiers see
    /// [`BindingBuilder::with_modifiers`](super::input_binding::BindingBuilder::with_modifiers).
    ///
    /// # Examples
    ///
    /// Single modifier:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_modifiers(Scale::splat(2.0));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_modifiers((Scale::splat(2.0), Negate::all()));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// For input-level conditions see
    /// [`BindingBuilder::with_conditions`](super::input_binding::BindingBuilder::with_conditions).
    ///
    /// # Examples
    ///
    /// Single condition:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_conditions(Release::default());
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .with_conditions((Release::default(), JustPress::default()));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// The action can be triggered by any input mapping. If multiple input mappings
    /// return [`ActionState`], the behavior is determined by [`InputAction::ACCUMULATION`].
    ///
    /// Thanks to traits, this function can be called with multiple types:
    ///
    /// 1. Raw input types.
    /// 2. [`Input`] enum which wraps any supported raw input and can store keyboard modifiers.
    /// 3. [`InputBinding`] which wraps [`Input`] and can store input modifiers or conditions.
    /// 4. [`IntoBindings`] which wraps [`InputBinding`] and can store multiple [`InputBinding`]s.
    ///    Also implemented on tuples, so you can pass multiple inputs to a single call.
    ///
    /// # Examples
    ///
    /// Raw input:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>()
    ///     .to((KeyCode::Space, GamepadButton::South));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>().to(KeyCode::Space.with_mod_keys(ModKeys::CONTROL));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Jump>().to(KeyCode::Space.with_conditions(Release::default()));
    /// actions.bind::<Attack>().to(MouseButton::Left.with_modifiers(Scale::splat(10.0)));
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Zoom>().to(Input::mouse_wheel());
    /// actions.bind::<Move>().to(Input::mouse_motion());
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// actions.bind::<Move>().to(Cardinal::wasd_keys());
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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
    /// # let mut actions = Actions::<Dummy>::default();
    /// # let mut settings = KeyboardSettings::default();
    /// actions.bind::<Inspect>().to(&settings.inspect);
    ///
    /// # #[derive(Default)]
    /// struct KeyboardSettings {
    ///     inspect: Vec<KeyCode>,
    /// }
    /// # #[derive(InputContext)]
    /// # struct Dummy;
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

    pub(crate) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        action_map: &mut ActionMap,
        time: &Time<Virtual>,
        entity: Entity,
    ) {
        trace!("updating action `{}`", self.action_name);

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

        let action = action_map
            .get_mut(&self.type_id)
            .expect("actions and bindings should have matching type IDs");

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

        action.update(time, state, value);
        if !tracker.events_blocked() {
            action.trigger_events(commands, entity);
        }
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
}
