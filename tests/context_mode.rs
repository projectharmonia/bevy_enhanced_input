mod action_recorder;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};
use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

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
    .record_action::<Consume>()
    .record_action::<Passthrough>();

    let entity1 = app.world_mut().spawn(Exclusive).id();
    let entity2 = app.world_mut().spawn(Exclusive).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(Consume::KEY);
    keys.press(Passthrough::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    assert_eq!(recorded.last::<Consume>(entity1).state, ActionState::Fired);
    assert_eq!(
        recorded.last::<Passthrough>(entity1).state,
        ActionState::Fired
    );
    assert!(
        recorded.is_empty::<Consume>(entity2),
        "only first entity with the same mappings that consume inputs should receive them"
    );
    assert_eq!(
        recorded.last::<Passthrough>(entity2).state,
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
    .record_action::<Consume>()
    .record_action::<Passthrough>();

    let entity1 = app.world_mut().spawn(Shared).id();
    let entity2 = app.world_mut().spawn(Shared).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(Consume::KEY);
    keys.press(Passthrough::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    assert_eq!(recorded.last::<Consume>(entity1).state, ActionState::Fired);
    assert_eq!(
        recorded.last::<Passthrough>(entity1).state,
        ActionState::Fired
    );
    assert_eq!(recorded.last::<Consume>(entity2).state, ActionState::Fired);
    assert_eq!(
        recorded.last::<Passthrough>(entity2).state,
        ActionState::Fired
    );
}

#[derive(Debug, Component)]
struct Exclusive;

impl InputContext for Exclusive {
    const MODE: ContextMode = ContextMode::Exclusive;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<Consume>().with(Consume::KEY);
        ctx.bind::<Passthrough>().with(Passthrough::KEY);
        ctx
    }
}

#[derive(Debug, Component)]
struct Shared;

impl InputContext for Shared {
    const MODE: ContextMode = ContextMode::Shared;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<Consume>().with(Consume::KEY);
        ctx.bind::<Passthrough>().with(Passthrough::KEY);
        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = true)]
struct Consume;

impl Consume {
    const KEY: KeyCode = KeyCode::Space;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool, consume_input = false)]
struct Passthrough;

impl Passthrough {
    const KEY: KeyCode = KeyCode::KeyE;
}
