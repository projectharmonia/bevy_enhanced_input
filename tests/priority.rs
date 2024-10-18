mod action_recorder;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn prioritization() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<First>()
    .add_input_context::<Second>()
    .record_action::<FirstConsume>()
    .record_action::<FirstPassthrough>()
    .record_action::<SecondConsume>()
    .record_action::<SecondPassthrough>();

    let entity = app.world_mut().spawn((First, Second)).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();

    let events = recorded.get::<FirstConsume>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<SecondConsume>(entity).unwrap();
    assert!(
        events.is_empty(),
        "action should be consumed by context with a higher priority"
    );

    let events = recorded.get::<FirstPassthrough>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.state, ActionState::Fired);

    let events = recorded.get::<SecondPassthrough>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(
        event.state,
        ActionState::Fired,
        "actions that doesn't consume inputs should still be triggered"
    );
}

/// A key used by both [`FirstConsume`] and [`SecondConsume`] actions.
const CONSUME_KEY: KeyCode = KeyCode::Space;

/// A key used by both [`FirstPassthrough`] and [`SecondPassthrough`] actions.
const PASSTHROUGH_KEY: KeyCode = KeyCode::KeyE;

#[derive(Debug, Component)]
struct First;

impl InputContext for First {
    const PRIORITY: usize = Second::PRIORITY + 1;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<FirstConsume>().with(CONSUME_KEY);
        ctx.bind::<FirstPassthrough>().with(PASSTHROUGH_KEY);
        ctx
    }
}

#[derive(Debug, Component)]
struct Second;

impl InputContext for Second {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<SecondConsume>().with(CONSUME_KEY);
        ctx.bind::<SecondPassthrough>().with(PASSTHROUGH_KEY);
        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = true)]
struct FirstConsume;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = true)]
struct SecondConsume;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = false)]
struct FirstPassthrough;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = false)]
struct SecondPassthrough;
