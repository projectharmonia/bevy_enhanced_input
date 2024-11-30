use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn passthrough() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<ConsumeThenPassthrough>();

    let entity = app.world_mut().spawn(ConsumeThenPassthrough).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<ConsumeThenPassthrough>(entity).unwrap();

    let action = ctx.action::<Consume>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let action = ctx.action::<Passthrough>().unwrap();
    assert_eq!(
        action.state(),
        ActionState::None,
        "action should be consumed"
    );
}

#[test]
fn consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<PassthroughThenConsume>();

    let entity = app.world_mut().spawn(PassthroughThenConsume).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<PassthroughThenConsume>(entity).unwrap();

    let action = ctx.action::<Consume>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let action = ctx.action::<Passthrough>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);
}

#[derive(Debug, Component)]
struct PassthroughThenConsume;

impl InputContext for PassthroughThenConsume {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Passthrough>().to(KEY);
        ctx.bind::<Consume>().to(KEY);

        ctx
    }
}

#[derive(Debug, Component)]
struct ConsumeThenPassthrough;

impl InputContext for ConsumeThenPassthrough {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Consume>().to(KEY);
        ctx.bind::<Passthrough>().to(KEY);

        ctx
    }
}

/// A key used by both [`Consume`] and [`Passthrough`] actions.
const KEY: KeyCode = KeyCode::KeyA;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct Consume;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct Passthrough;
