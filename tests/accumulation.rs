mod action_recorder;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};
use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn max_abs() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<Moving>()
    .record_action::<MaxAbsMove>();

    let entity = app.world_mut().spawn(Moving).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    assert_eq!(recorded.last::<MaxAbsMove>(entity).value, Vec2::Y.into());
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
    .add_input_context::<Moving>()
    .record_action::<CumulativeMove>();

    let entity = app.world_mut().spawn(Moving).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowDown);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    assert!(
        recorded.is_empty::<CumulativeMove>(entity),
        "up and down should cancel each other"
    );
}

#[derive(Debug, Component)]
struct Moving;

impl InputContext for Moving {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<MaxAbsMove>().with_wasd();
        ctx.bind::<CumulativeMove>().with_arrows();

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D, accumulation = MaxAbs)]
struct MaxAbsMove;

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D, accumulation = Cumulative)]
struct CumulativeMove;
