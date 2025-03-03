use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<ConsumeOnly>()
        .add_observer(consume_only_binding);

    let entity1 = app.world_mut().spawn(ConsumeOnly).id();
    let entity2 = app.world_mut().spawn(ConsumeOnly).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();

    let entity1_ctx = registry.context::<ConsumeOnly>(entity1);
    assert_eq!(entity1_ctx.action::<Consume>().state(), ActionState::Fired);

    let entity2_ctx = registry.context::<ConsumeOnly>(entity2);
    assert_eq!(
        entity2_ctx.action::<Consume>().state(),
        ActionState::None,
        "only first entity with the same mappings that consume inputs should receive them"
    );
}

#[test]
fn passthrough() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<PassthroughOnly>()
        .add_observer(passthrough_only_binding);

    let entity1 = app.world_mut().spawn(PassthroughOnly).id();
    let entity2 = app.world_mut().spawn(PassthroughOnly).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();

    let entity1_ctx = registry.context::<PassthroughOnly>(entity1);
    assert_eq!(
        entity1_ctx.action::<Passthrough>().state(),
        ActionState::Fired
    );

    let entity2_ctx = registry.context::<PassthroughOnly>(entity2);
    assert_eq!(
        entity2_ctx.action::<Passthrough>().state(),
        ActionState::Fired,
        "actions that doesn't consume inputs should still fire"
    );
}

#[test]
fn consume_then_passthrough() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<ConsumeThenPassthrough>()
        .add_observer(consume_then_passthrough_binding);

    let entity = app.world_mut().spawn(ConsumeThenPassthrough).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<ConsumeThenPassthrough>(entity);
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
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<PassthroughThenConsume>()
        .add_observer(passthrough_then_consume_binding);

    let entity = app.world_mut().spawn(PassthroughThenConsume).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<PassthroughThenConsume>(entity);
    assert_eq!(ctx.action::<Consume>().state(), ActionState::Fired);
    assert_eq!(ctx.action::<Passthrough>().state(), ActionState::Fired);
}

fn consume_only_binding(mut trigger: Trigger<Binding<ConsumeOnly>>) {
    trigger.bind::<Consume>().to(KEY);
}

fn passthrough_only_binding(mut trigger: Trigger<Binding<PassthroughOnly>>) {
    trigger.bind::<Passthrough>().to(KEY);
}

fn consume_then_passthrough_binding(mut trigger: Trigger<Binding<ConsumeThenPassthrough>>) {
    trigger.bind::<Consume>().to(KEY);
    trigger.bind::<Passthrough>().to(KEY);
}

fn passthrough_then_consume_binding(mut trigger: Trigger<Binding<PassthroughThenConsume>>) {
    trigger.bind::<Passthrough>().to(KEY);
    trigger.bind::<Consume>().to(KEY);
}

#[derive(Debug, Component)]
struct PassthroughOnly;

#[derive(Debug, Component)]
struct ConsumeOnly;

#[derive(Debug, Component)]
struct PassthroughThenConsume;

#[derive(Debug, Component)]
struct ConsumeThenPassthrough;

/// A key used by both [`Consume`] and [`Passthrough`] actions.
const KEY: KeyCode = KeyCode::KeyA;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct Consume;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct Passthrough;
