mod action_recorder;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn exclusive() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<Exclusive>()
    .record_action::<ExclusiveConsume>()
    .record_action::<ExclusivePassthrough>();

    let entity1 = app.world_mut().spawn(Exclusive).id();
    let entity2 = app.world_mut().spawn(Exclusive).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ExclusiveConsume::KEY);
    keys.press(ExclusivePassthrough::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ExclusiveConsume>(entity1).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<ExclusivePassthrough>(entity1).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<ExclusiveConsume>(entity2).unwrap();
    assert!(
        events.is_empty(),
        "only first entity with the same mappings that consume inputs should receive them"
    );

    let events = recorded.get::<ExclusivePassthrough>(entity2).unwrap();
    let event = events.last().unwrap();
    assert_eq!(
        event.state,
        ActionState::Fired,
        "actions that doesn't consume inputs should still fire"
    );
}

#[test]
fn shared() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<Shared>()
    .record_action::<SharedConsume>()
    .record_action::<SharedPassthrough>();

    let entity1 = app.world_mut().spawn(Shared).id();
    let entity2 = app.world_mut().spawn(Shared).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(SharedConsume::KEY);
    keys.press(SharedPassthrough::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<SharedConsume>(entity1).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<SharedPassthrough>(entity1).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<SharedConsume>(entity2).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<SharedPassthrough>(entity2).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);
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
    .add_input_context::<Exclusive>()
    .add_input_context::<Shared>()
    .record_action::<ExclusiveConsume>()
    .record_action::<SharedConsume>();

    let entity = app.world_mut().spawn((Exclusive, Shared)).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ExclusiveConsume::KEY);
    keys.press(SharedConsume::KEY);

    app.update();

    app.world_mut()
        .entity_mut(entity)
        .remove::<Exclusive>()
        .remove::<Shared>();

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ExclusiveConsume>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.kind.is_completed());

    let events = recorded.get::<SharedConsume>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.kind.is_completed());
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
    .add_input_context::<Exclusive>()
    .add_input_context::<Shared>()
    .record_action::<ExclusiveConsume>()
    .record_action::<SharedConsume>();

    let entity = app.world_mut().spawn((Exclusive, Shared)).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ExclusiveConsume::KEY);
    keys.press(SharedConsume::KEY);

    app.update();

    app.world_mut().trigger(RebuildInputContexts);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<ExclusiveConsume>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.kind.is_completed());

    let events = recorded.get::<SharedConsume>(entity).unwrap();
    let [event] = events.try_into().unwrap();
    assert!(event.kind.is_completed());
}

#[derive(Debug, Component)]
struct Exclusive;

impl InputContext for Exclusive {
    const MODE: ContextMode = ContextMode::Exclusive;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<ExclusiveConsume>().with(ExclusiveConsume::KEY);
        ctx.bind::<ExclusivePassthrough>()
            .with(ExclusivePassthrough::KEY);
        ctx
    }
}

#[derive(Debug, Component)]
struct Shared;

impl InputContext for Shared {
    const MODE: ContextMode = ContextMode::Shared;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<SharedConsume>().with(SharedConsume::KEY);
        ctx.bind::<SharedPassthrough>().with(SharedPassthrough::KEY);
        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = true)]
struct ExclusiveConsume;

impl ExclusiveConsume {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = false)]
struct ExclusivePassthrough;

impl ExclusivePassthrough {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = true)]
struct SharedConsume;

impl SharedConsume {
    const KEY: KeyCode = KeyCode::KeyC;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = false)]
struct SharedPassthrough;

impl SharedPassthrough {
    const KEY: KeyCode = KeyCode::KeyD;
}
