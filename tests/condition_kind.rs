mod action_recorder;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn explicit() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<Explicit>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<Explicit>(entity).unwrap();
    assert!(events.is_empty());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Explicit::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<Explicit>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Explicit::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<Explicit>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::None);
}

#[test]
fn implicit() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<ReleaseAction>()
    .record_action::<Implicit>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    assert!(events.is_empty());

    let events = recorded.get::<Implicit>(entity).unwrap();
    assert!(events.is_empty());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ReleaseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Ongoing);

    let events = recorded.get::<Implicit>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(ReleaseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<Implicit>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::Fired);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::None);

    let events = recorded.get::<Implicit>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::None);
}

#[test]
fn blocker() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<ReleaseAction>()
    .record_action::<Blocker>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    assert!(events.is_empty());

    let events = recorded.get::<Blocker>(entity).unwrap();
    assert!(events.is_empty());

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ReleaseAction::KEY);
    keys.press(Blocker::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Ongoing);

    let events = recorded.get::<Blocker>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(ReleaseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<Blocker>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::None);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::None);

    let events = recorded.get::<Blocker>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Fired);
}

#[test]
fn events_blocker() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<ReleaseAction>()
    .record_action::<EventsBlocker>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    assert!(events.is_empty());

    let events = recorded.get::<EventsBlocker>(entity).unwrap();
    assert!(events.is_empty());

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ReleaseAction::KEY);
    keys.press(EventsBlocker::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Ongoing);

    let events = recorded.get::<EventsBlocker>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(ReleaseAction::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<EventsBlocker>(entity).unwrap();
    assert!(events.is_empty());

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ReleaseAction>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, false.into());
    assert_eq!(event.state, ActionState::None);

    let events = recorded.get::<EventsBlocker>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, true.into());
    assert_eq!(event.state, ActionState::Fired);
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<ReleaseAction>()
            .with(ReleaseAction::KEY)
            .with_condition(Release::default());
        ctx.bind::<Explicit>()
            .with_condition(Down::default())
            .with(Explicit::KEY);
        ctx.bind::<Implicit>()
            .with_condition(Chord::<ReleaseAction>::default());
        ctx.bind::<Blocker>()
            .with(Blocker::KEY)
            .with_condition(BlockBy::<ReleaseAction>::default());
        ctx.bind::<EventsBlocker>()
            .with(EventsBlocker::KEY)
            .with_condition(BlockBy::<ReleaseAction>::events_only());

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct ReleaseAction;

impl ReleaseAction {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Explicit;

impl Explicit {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Implicit;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Blocker;

impl Blocker {
    const KEY: KeyCode = KeyCode::KeyD;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct EventsBlocker;

impl EventsBlocker {
    const KEY: KeyCode = KeyCode::KeyE;
}
