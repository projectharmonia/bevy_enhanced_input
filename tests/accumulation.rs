mod action_recorder;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn max_abs() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<MaxAbs>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<MaxAbs>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, Vec2::Y.into());
}

#[test]
fn cumulative() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<Cumulative>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowDown);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<Cumulative>(entity).unwrap();
    assert!(events.is_empty(), "up and down should cancel each other");
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<MaxAbs>().with_wasd();
        // ctx.bind::<Cumulative>().with_arrows();

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D, accumulation = MaxAbs)]
struct MaxAbs;

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D, accumulation = Cumulative)]
struct Cumulative;
