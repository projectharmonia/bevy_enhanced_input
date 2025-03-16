use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn max_abs() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>()
        .add_observer(binding);

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<DummyContext>(entity);
    assert_eq!(ctx.action::<MaxAbs>().value(), Vec2::Y.into());
}

#[test]
fn cumulative() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>()
        .add_observer(binding);

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowDown);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<DummyContext>(entity);
    assert_eq!(
        ctx.action::<Cumulative>().value(),
        Vec2::ZERO.into(),
        "up and down should cancel each other"
    );
}

fn binding(mut trigger: Trigger<Binding<DummyContext>>) {
    trigger.bind::<MaxAbs>().to(Cardinal::wasd_keys());
    trigger.bind::<Cumulative>().to(Cardinal::arrow_keys());
}

#[derive(Debug, Component)]
struct DummyContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, accumulation = MaxAbs)]
struct MaxAbs;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, accumulation = Cumulative)]
struct Cumulative;
