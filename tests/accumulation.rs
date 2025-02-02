use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn max_abs() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.context::<DummyContext>(entity);
    assert_eq!(ctx.action::<MaxAbs>().value(), Vec2::Y.into());
}

#[test]
fn cumulative() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowDown);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.context::<DummyContext>(entity);
    assert_eq!(
        ctx.action::<Cumulative>().value(),
        Vec2::ZERO.into(),
        "up and down should cancel each other"
    );
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<MaxAbs>().to(Cardinal::wasd_keys());
        ctx.bind::<Cumulative>().to(Cardinal::arrow_keys());

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, accumulation = MaxAbs)]
struct MaxAbs;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, accumulation = Cumulative)]
struct Cumulative;
