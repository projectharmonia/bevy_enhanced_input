mod action_recorder;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn fired_completed() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<PressAction>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(PressAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PressAction>(entity).unwrap();
    let [event1, event2] = events.try_into().unwrap();
    assert!(event1.transition.is_started());
    assert!(event2.transition.is_fired());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PressAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_fired());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(PressAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PressAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_completed());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PressAction>(entity).unwrap();
    assert!(events.is_empty());
}

#[test]
fn ongoing_fired_completed() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<ReleaseAction>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ReleaseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let [event1, event2] = events.try_into().unwrap();
    assert!(event1.transition.is_started());
    assert!(event2.transition.is_ongoing());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(ReleaseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_fired());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_completed());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    assert!(events.is_empty());
}

#[test]
fn ongoing_cancelled() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<HoldAction>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(HoldAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<HoldAction>(entity).unwrap();
    let [event1, event2] = events.try_into().unwrap();
    assert!(event1.transition.is_started());
    assert!(event2.transition.is_ongoing());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(HoldAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<HoldAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_canceled());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<HoldAction>(entity).unwrap();
    assert!(events.is_empty());
}

#[test]
fn fired_ongoing_cancelled() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<PulseAction>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(PulseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PulseAction>(entity).unwrap();
    let [event1, event2] = events.try_into().unwrap();
    assert!(event1.transition.is_started());
    assert!(event2.transition.is_fired());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PulseAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_ongoing());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(PulseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PulseAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_canceled());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<PulseAction>(entity).unwrap();
    assert!(events.is_empty());
}

#[test]
fn context_removal() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<ReleaseAction>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ReleaseAction::KEY);

    app.update();

    app.world_mut().entity_mut(entity).remove::<DummyContext>();

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_canceled());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    assert!(events.is_empty());
}

#[test]
fn context_rebuild() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<ReleaseAction>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ReleaseAction::KEY);

    app.update();

    app.world_mut().trigger(RebuildInputContexts);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.transition.is_canceled());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    assert!(events.is_empty());
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<PressAction>().with(PressAction::KEY);
        ctx.bind::<ReleaseAction>()
            .with(ReleaseAction::KEY)
            .with_condition(Released::default());
        ctx.bind::<HoldAction>()
            .with(HoldAction::KEY)
            .with_condition(Hold::new(1.0));
        ctx.bind::<PulseAction>()
            .with(PulseAction::KEY)
            .with_condition(Pulse::new(1.0));

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct PressAction;

impl PressAction {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct ReleaseAction;

impl ReleaseAction {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct HoldAction;

impl HoldAction {
    const KEY: KeyCode = KeyCode::KeyC;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct PulseAction;

impl PulseAction {
    const KEY: KeyCode = KeyCode::KeyD;
}
