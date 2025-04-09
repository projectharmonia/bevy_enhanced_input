use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<ConsumeOnly>()
        .add_observer(consume_only_binding)
        .finish();

    let entity1 = app
        .world_mut()
        .spawn(Actions::<ConsumeOnly>::default())
        .id();
    let entity2 = app
        .world_mut()
        .spawn(Actions::<ConsumeOnly>::default())
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let entity1_ctx = app.world().get::<Actions<ConsumeOnly>>(entity1).unwrap();
    assert_eq!(entity1_ctx.action::<Consume>().state(), ActionState::Fired);

    let entity2_ctx = app.world().get::<Actions<ConsumeOnly>>(entity2).unwrap();
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
        .add_observer(passthrough_only_binding)
        .finish();

    let entity1 = app
        .world_mut()
        .spawn(Actions::<PassthroughOnly>::default())
        .id();
    let entity2 = app
        .world_mut()
        .spawn(Actions::<PassthroughOnly>::default())
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let entity1_ctx = app
        .world()
        .get::<Actions<PassthroughOnly>>(entity1)
        .unwrap();
    assert_eq!(
        entity1_ctx.action::<Passthrough>().state(),
        ActionState::Fired
    );

    let entity2_ctx = app
        .world()
        .get::<Actions<PassthroughOnly>>(entity2)
        .unwrap();
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
        .add_observer(consume_then_passthrough_binding)
        .finish();

    let entity = app
        .world_mut()
        .spawn(Actions::<ConsumeThenPassthrough>::default())
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let actions = app
        .world()
        .get::<Actions<ConsumeThenPassthrough>>(entity)
        .unwrap();
    assert_eq!(actions.action::<Consume>().state(), ActionState::Fired);
    assert_eq!(
        actions.action::<Passthrough>().state(),
        ActionState::None,
        "action should be consumed"
    );
}

#[test]
fn passthrough_then_consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<PassthroughThenConsume>()
        .add_observer(passthrough_then_consume_binding)
        .finish();

    let entity = app
        .world_mut()
        .spawn(Actions::<PassthroughThenConsume>::default())
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let actions = app
        .world()
        .get::<Actions<PassthroughThenConsume>>(entity)
        .unwrap();
    assert_eq!(actions.action::<Consume>().state(), ActionState::Fired);
    assert_eq!(actions.action::<Passthrough>().state(), ActionState::Fired);
}

fn consume_only_binding(
    trigger: Trigger<Binding<ConsumeOnly>>,
    mut actions: Query<&mut Actions<ConsumeOnly>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Consume>().to(KEY);
}

fn passthrough_only_binding(
    trigger: Trigger<Binding<PassthroughOnly>>,
    mut actions: Query<&mut Actions<PassthroughOnly>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Passthrough>().to(KEY);
}

fn consume_then_passthrough_binding(
    trigger: Trigger<Binding<ConsumeThenPassthrough>>,
    mut actions: Query<&mut Actions<ConsumeThenPassthrough>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Consume>().to(KEY);
    actions.bind::<Passthrough>().to(KEY);
}

fn passthrough_then_consume_binding(
    trigger: Trigger<Binding<PassthroughThenConsume>>,
    mut actions: Query<&mut Actions<PassthroughThenConsume>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Passthrough>().to(KEY);
    actions.bind::<Consume>().to(KEY);
}

#[derive(InputContext)]
struct PassthroughOnly;

#[derive(InputContext)]
struct ConsumeOnly;

#[derive(InputContext)]
struct PassthroughThenConsume;

#[derive(InputContext)]
struct ConsumeThenPassthrough;

/// A key used by both [`Consume`] and [`Passthrough`] actions.
const KEY: KeyCode = KeyCode::KeyA;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct Consume;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct Passthrough;
