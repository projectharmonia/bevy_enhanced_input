mod action_recorder;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};
use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

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
    assert_eq!(
        recorded.last::<FirstConsume>(entity).state,
        ActionState::Fired
    );
    assert!(
        recorded.is_empty::<SecondConsume>(entity),
        "action should be consumed by context with a higher priority"
    );
    assert_eq!(
        recorded.last::<FirstPassthrough>(entity).state,
        ActionState::Fired,
    );
    assert_eq!(
        recorded.last::<SecondPassthrough>(entity).state,
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
