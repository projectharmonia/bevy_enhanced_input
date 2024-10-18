mod action_recorder;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn passthrough() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<ConsumeThenPassthrough>()
    .record_action::<Consume>()
    .record_action::<Passthrough>();

    let entity = app.world_mut().spawn(ConsumeThenPassthrough).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<Consume>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<Passthrough>(entity).unwrap();
    assert!(events.is_empty(), "action should be consumed");
}

#[test]
fn consume() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<PassthroughThenConsume>()
    .record_action::<Consume>()
    .record_action::<Passthrough>();

    let entity = app.world_mut().spawn(PassthroughThenConsume).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<Consume>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<Passthrough>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);
}

/// A key used by both [`Consume`] and [`Passthrough`] actions.
const KEY: KeyCode = KeyCode::Space;

#[derive(Debug, Component)]
struct PassthroughThenConsume;

impl InputContext for PassthroughThenConsume {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Passthrough>().with(KEY);
        ctx.bind::<Consume>().with(KEY);

        ctx
    }
}

#[derive(Debug, Component)]
struct ConsumeThenPassthrough;

impl InputContext for ConsumeThenPassthrough {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Consume>().with(KEY);
        ctx.bind::<Passthrough>().with(KEY);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = true)]
struct Consume;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = false)]
struct Passthrough;
