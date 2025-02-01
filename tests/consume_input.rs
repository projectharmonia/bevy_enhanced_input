use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins))
        .add_input_context::<ConsumeOnly>();

    let entity1 = app.world_mut().spawn(ConsumeOnly).id();
    let entity2 = app.world_mut().spawn(ConsumeOnly).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();

    let entity1_ctx = instances.context::<ConsumeOnly>(entity1);
    assert_eq!(entity1_ctx.action::<Consume>().state(), ActionState::Fired);

    let entity2_ctx = instances.context::<ConsumeOnly>(entity2);
    assert_eq!(
        entity2_ctx.action::<Consume>().state(),
        ActionState::None,
        "only first entity with the same mappings that consume inputs should receive them"
    );
}

#[test]
fn passthrough() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins))
        .add_input_context::<PassthroughOnly>();

    let entity1 = app.world_mut().spawn(PassthroughOnly).id();
    let entity2 = app.world_mut().spawn(PassthroughOnly).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();

    let entity1_ctx = instances.context::<PassthroughOnly>(entity1);
    assert_eq!(
        entity1_ctx.action::<Passthrough>().state(),
        ActionState::Fired
    );

    let entity2_ctx = instances.context::<PassthroughOnly>(entity2);
    assert_eq!(
        entity2_ctx.action::<Passthrough>().state(),
        ActionState::Fired,
        "actions that doesn't consume inputs should still fire"
    );
}

#[test]
fn consume_then_passthrough() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins))
        .add_input_context::<ConsumeThenPassthrough>();

    let entity = app.world_mut().spawn(ConsumeThenPassthrough).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.context::<ConsumeThenPassthrough>(entity);
    assert_eq!(ctx.action::<Consume>().state(), ActionState::Fired);
    assert_eq!(
        ctx.action::<Passthrough>().state(),
        ActionState::None,
        "action should be consumed"
    );
}

#[test]
fn passthrough_then_consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins))
        .add_input_context::<PassthroughThenConsume>();

    let entity = app.world_mut().spawn(PassthroughThenConsume).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.context::<PassthroughThenConsume>(entity);
    assert_eq!(ctx.action::<Consume>().state(), ActionState::Fired);
    assert_eq!(ctx.action::<Passthrough>().state(), ActionState::Fired);
}

#[derive(Debug, Component)]
struct PassthroughOnly;

impl InputContext for PassthroughOnly {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<Passthrough>().to(KEY);
        ctx
    }
}

#[derive(Debug, Component)]
struct ConsumeOnly;

impl InputContext for ConsumeOnly {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<Consume>().to(KEY);
        ctx
    }
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
