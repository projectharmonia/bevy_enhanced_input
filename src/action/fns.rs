use core::{any, fmt::Debug};

use bevy::prelude::*;
use log::debug;

use crate::prelude::*;

#[derive(Component, Clone, Copy)]
#[component(immutable)]
pub(crate) struct ActionFns {
    store_value: fn(&mut EntityMut, ActionValue),
    trigger: fn(&mut Commands, Entity, ActionState, ActionEvents, ActionValue, ActionTime),
}

impl ActionFns {
    pub(super) fn new<A: InputAction>() -> Self {
        Self {
            store_value: store_value::<A>,
            trigger: trigger::<A>,
        }
    }

    pub(crate) fn store_value(&self, entity: &mut EntityMut, value: ActionValue) {
        (self.store_value)(entity, value);
    }

    pub(crate) fn trigger(
        &self,
        commands: &mut Commands,
        entity: Entity,
        state: ActionState,
        events: ActionEvents,
        value: ActionValue,
        time: ActionTime,
    ) {
        (self.trigger)(commands, entity, state, events, value, time);
    }
}

fn store_value<A: InputAction>(entity: &mut EntityMut, value: ActionValue) {
    let mut action = entity
        .get_mut::<Action<A>>()
        .expect("entity should be an action");

    **action = A::Output::unwrap_value(value);
}

fn trigger<A: InputAction>(
    commands: &mut Commands,
    entity: Entity,
    state: ActionState,
    events: ActionEvents,
    value: ActionValue,
    time: ActionTime,
) {
    for (_, event) in events.iter_names() {
        match event {
            ActionEvents::STARTED => {
                trigger_and_log::<A, _>(
                    commands,
                    entity,
                    Started::<A> {
                        value: A::Output::unwrap_value(value),
                        state,
                    },
                );
            }
            ActionEvents::ONGOING => {
                trigger_and_log::<A, _>(
                    commands,
                    entity,
                    Ongoing::<A> {
                        value: A::Output::unwrap_value(value),
                        state,
                        elapsed_secs: time.elapsed_secs,
                    },
                );
            }
            ActionEvents::FIRED => {
                trigger_and_log::<A, _>(
                    commands,
                    entity,
                    Fired::<A> {
                        value: A::Output::unwrap_value(value),
                        state,
                        fired_secs: time.fired_secs,
                        elapsed_secs: time.elapsed_secs,
                    },
                );
            }
            ActionEvents::CANCELED => {
                trigger_and_log::<A, _>(
                    commands,
                    entity,
                    Canceled::<A> {
                        value: A::Output::unwrap_value(value),
                        state,
                        elapsed_secs: time.elapsed_secs,
                    },
                );
            }
            ActionEvents::COMPLETED => {
                trigger_and_log::<A, _>(
                    commands,
                    entity,
                    Completed::<A> {
                        value: A::Output::unwrap_value(value),
                        state,
                        fired_secs: time.fired_secs,
                        elapsed_secs: time.elapsed_secs,
                    },
                );
            }
            _ => unreachable!("iteration should yield only named flags"),
        }
    }
}

fn trigger_and_log<A, E: Event + Debug>(commands: &mut Commands, entity: Entity, event: E) {
    debug!(
        "triggering `{event:?}` for `{}` for `{entity}`",
        any::type_name::<A>()
    );
    commands.trigger_targets(event, entity);
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;
    use test_log::test;

    use super::*;

    #[test]
    fn none_none() {
        let events = transition(ActionState::None, ActionState::None);
        assert!(events.is_empty());
    }

    #[test]
    fn none_ongoing() {
        let events = transition(ActionState::None, ActionState::Ongoing);
        assert_eq!(events, ActionEvents::STARTED | ActionEvents::ONGOING);
    }

    #[test]
    fn none_fired() {
        let events = transition(ActionState::None, ActionState::Fired);
        assert_eq!(events, ActionEvents::STARTED | ActionEvents::FIRED);
    }

    #[test]
    fn ongoing_none() {
        let events = transition(ActionState::Ongoing, ActionState::None);
        assert_eq!(events, ActionEvents::CANCELED);
    }

    #[test]
    fn ongoing_ongoing() {
        let events = transition(ActionState::Ongoing, ActionState::Ongoing);
        assert_eq!(events, ActionEvents::ONGOING);
    }

    #[test]
    fn ongoing_fired() {
        let events = transition(ActionState::Ongoing, ActionState::Fired);
        assert_eq!(events, ActionEvents::FIRED);
    }

    #[test]
    fn fired_none() {
        let events = transition(ActionState::Fired, ActionState::None);
        assert_eq!(events, ActionEvents::COMPLETED);
    }

    #[test]
    fn fired_ongoing() {
        let events = transition(ActionState::Fired, ActionState::Ongoing);
        assert_eq!(events, ActionEvents::ONGOING);
    }

    #[test]
    fn fired_fired() {
        let events = transition(ActionState::Fired, ActionState::Fired);
        assert_eq!(events, ActionEvents::FIRED);
    }

    fn transition(initial_state: ActionState, target_state: ActionState) -> ActionEvents {
        let mut world = World::new();

        world.init_resource::<TriggeredEvents>();
        world.add_observer(
            |_trigger: Trigger<Fired<TestAction>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::FIRED);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Started<TestAction>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::STARTED);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Ongoing<TestAction>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::ONGOING);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Completed<TestAction>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::COMPLETED);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Canceled<TestAction>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::CANCELED);
            },
        );

        let events = ActionEvents::new(initial_state, target_state);
        let fns = ActionFns::new::<TestAction>();
        fns.trigger(
            &mut world.commands(),
            Entity::PLACEHOLDER,
            target_state,
            events,
            false.into(),
            Default::default(),
        );

        world.flush();

        *world.remove_resource::<TriggeredEvents>().unwrap()
    }

    #[derive(Resource, Default, Deref, DerefMut)]
    struct TriggeredEvents(ActionEvents);

    #[derive(InputAction)]
    #[action_output(bool)]
    struct TestAction;
}
