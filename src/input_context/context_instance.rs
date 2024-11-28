use std::any::{self, TypeId};

use bevy::{prelude::*, utils::Entry};

use super::{
    bind::{InputBindSet, IntoInputBinds},
    input_action::{Accumulation, ActionData, ActionOutput, ActionsData, InputAction},
    input_condition::InputCondition,
    input_modifier::InputModifier,
    trigger_tracker::TriggerTracker,
};
use crate::{
    action_value::{ActionValue, ActionValueDim},
    input::{input_reader::InputReader, GamepadDevice, Input},
    ActionState,
};

/// Instance for [`InputContext`](super::InputContext).
///
/// Stores [`InputAction`]s and evaluates their [`ActionState`] in the order they are bound.
///
/// Each action can have multiple associated [`Input`]s, any of which can trigger the action.
///
/// Additionally, you can assign [`InputModifier`]s and [`InputCondition`]s at both the action
/// and input levels.
///
/// Action evaluation follows these steps:
///
/// 1. Iterate over each [`ActionValue`] from the associated [`Input`]s:
///    1.1. Apply input-level [`InputModifier`]s.
///    1.2. Evaluate input-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
/// 2. Select all [`ActionValue`]s with the most significant [`ActionState`] and combine based on [`InputAction::ACCUMULATION`].
///    Combined value be converted into [`ActionOutput::DIM`] using [`ActionValue::convert`].
/// 3. Apply action level [`InputModifier`]s.
/// 4. Evaluate action level [`InputCondition`]s, combining their results according to [`InputCondition::kind`].
/// 5. Set the final [`ActionState`] based on the results.
///    Final value be converted into [`InputAction::Output`] using [`ActionOutput::convert_from`].
///
/// New instances won't react to currently held inputs until they are released.
/// This prevents unintended behavior where switching contexts using the same key
/// could cause an immediate switch back, as buttons are rarely pressed for only a single frame.
///
/// [`ActionState`]: super::input_action::ActionState
#[derive(Default)]
pub struct ContextInstance {
    gamepad: GamepadDevice,
    bindings: Vec<ActionBind>,
    actions: ActionsData,
}

impl ContextInstance {
    /// Associates context with gamepad.
    ///
    /// By default it's [`GamepadDevice::Any`].
    pub fn with_gamepad(gamepad: impl Into<GamepadDevice>) -> Self {
        Self {
            gamepad: gamepad.into(),
            ..Default::default()
        }
    }

    /// Starts binding an action.
    ///
    /// This method can be called multiple times for the same action to extend its mappings.
    pub fn bind<A: InputAction>(&mut self) -> &mut ActionBind {
        let type_id = TypeId::of::<A>();
        match self.actions.entry(type_id) {
            Entry::Occupied(_entry) => self
                .bindings
                .iter_mut()
                .find(|binding| binding.type_id == type_id)
                .expect("actions and bindings should have matching type IDs"),
            Entry::Vacant(entry) => {
                entry.insert(ActionData::new::<A>());
                self.bindings.push(ActionBind::new::<A>());
                self.bindings.last_mut().unwrap()
            }
        }
    }

    /// Returns associated state for action `A`.
    ///
    /// See also [`ContextInstances::get`](super::ContextInstances::get).
    pub fn action<A: InputAction>(&self) -> Option<&ActionData> {
        self.actions.action::<A>()
    }

    pub(super) fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        time: &Time<Virtual>,
        entities: &[Entity],
    ) {
        reader.set_gamepad(self.gamepad);
        for binding in &mut self.bindings {
            binding.update(commands, reader, &mut self.actions, time, entities);
        }
    }

    /// Copies [`ActionData`] for each binding and triggers transition to [`ActionState::None`] with zero value.
    ///
    /// Instance data remains unchanges.
    pub(super) fn trigger_removed(
        &self,
        commands: &mut Commands,
        time: &Time<Virtual>,
        entities: &[Entity],
    ) {
        for binding in &self.bindings {
            let mut action = *self
                .actions
                .get(&binding.type_id)
                .expect("actions and bindings should have matching type IDs");
            action.update(time, ActionState::None, ActionValue::zero(binding.dim));
            action.trigger_events(commands, entities);
        }
    }
}

/// Bindings of [`InputAction`] for [`ContextInstance`].
///
/// These bindings are stored separately from [`ActionsData`] to allow a currently
/// evaluating action to access the state of other actions.
pub struct ActionBind {
    type_id: TypeId,
    action_name: &'static str,
    consume_input: bool,
    accumulation: Accumulation,
    dim: ActionValueDim,
    config: InputBindSet,

    /// Consumed inputs during state evaluation.
    consume_buffer: Vec<Input>,
}

impl ActionBind {
    #[must_use]
    fn new<A: InputAction>() -> Self {
        Self {
            type_id: TypeId::of::<A>(),
            action_name: any::type_name::<A>(),
            dim: A::Output::DIM,
            consume_input: A::CONSUME_INPUT,
            accumulation: A::ACCUMULATION,
            config: InputBindSet::default(),
            consume_buffer: Default::default(),
        }
    }

    /// Adds action-level modifier.
    pub fn with_modifier(&mut self, modifier: impl InputModifier) -> &mut Self {
        debug!("adding `{modifier:?}` to `{}`", self.action_name);
        self.config.modifiers.push(Box::new(modifier));
        self
    }

    /// Adds action-level condition.
    pub fn with_condition(&mut self, condition: impl InputCondition) -> &mut Self {
        debug!("adding `{condition:?}` to `{}`", self.action_name);
        self.config.conditions.push(Box::new(condition));
        self
    }

    /// Adds input mapping.
    ///
    /// The action can be triggered by any input mapping. If multiple input mappings
    /// return [`ActionState`].
    ///
    /// Thanks to [`Into`] impls, it can be called directly with buttons/axes:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(KeyCode::Space)
    ///     .to(GamepadButtonType::South);
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// or with [`Input`] if you want to assign keyboard modifiers:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>().to(Input::Keyboard {
    ///     key: KeyCode::Space,
    ///     mod_keys: ModKeys::CONTROL,
    /// });
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    ///
    /// If you want input with modifiers or conditions,
    /// you will need to wrap it into [`InputBind`]:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut ctx = ContextInstance::default();
    /// ctx.bind::<Jump>()
    ///     .to(InputBind::new(KeyCode::Space).with_condition(Release::default()));
    /// # #[derive(Debug, InputAction)]
    /// # #[input_action(output = bool)]
    /// # struct Jump;
    /// ```
    pub fn to(&mut self, configs: impl IntoInputBinds) -> &mut Self {
        let configs = configs.into_binds();
        debug!("adding `{configs:?}` to `{}`", self.action_name);
        self.config.binds.push(configs);
        self
    }

    fn update(
        &mut self,
        commands: &mut Commands,
        reader: &mut InputReader,
        actions: &mut ActionsData,
        time: &Time<Virtual>,
        entities: &[Entity],
    ) {
        trace!("updating action `{}`", self.action_name);

        let mut tracker = TriggerTracker::new(ActionValue::zero(self.dim));

        // TODO
        // for binding in &mut self.bindings {
        //     let value = reader.value(binding.input);
        //     if binding.ignored {
        //         // Ignore until we read zero for this mapping.
        //         if value.as_bool() {
        //             continue;
        //         } else {
        //             binding.ignored = false;
        //         }
        //     }

        //     let mut current_tracker = TriggerTracker::new(value);
        //     current_tracker.apply_modifiers(actions, time, &mut binding.modifiers);
        //     current_tracker.apply_conditions(actions, time, &mut binding.conditions);

        //     let current_state = current_tracker.state();
        //     if current_state == ActionState::None {
        //         // Ignore non-active trackers to allow the action to fire even if all
        //         // input-level conditions return `ActionState::None`. This ensures that an
        //         // action-level condition or modifier can still trigger the action.
        //         continue;
        //     }

        //     match current_state.cmp(&tracker.state()) {
        //         Ordering::Less => (),
        //         Ordering::Equal => {
        //             tracker.combine(current_tracker, self.accumulation);
        //             if self.consume_input {
        //                 self.consume_buffer.push(binding.input);
        //             }
        //         }
        //         Ordering::Greater => {
        //             tracker.overwrite(current_tracker);
        //             if self.consume_input {
        //                 self.consume_buffer.clear();
        //                 self.consume_buffer.push(binding.input);
        //             }
        //         }
        //     }
        // }

        // tracker.apply_modifiers(actions, time, &mut self.modifiers);
        // tracker.apply_conditions(actions, time, &mut self.conditions);

        let action = actions
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
            action.trigger_events(commands, entities);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;

    #[test]
    fn bind() {
        let mut ctx = ContextInstance::default();
        ctx.bind::<DummyAction>().to(KeyCode::KeyA);
        ctx.bind::<DummyAction>().to(KeyCode::KeyB);
        assert_eq!(ctx.bindings.len(), 1);

        let action = ctx.bindings.first().unwrap();
        assert_eq!(action.config.binds.len(), 2);
    }

    #[derive(Debug, InputAction)]
    #[input_action(output = bool)]
    struct DummyAction;
}
