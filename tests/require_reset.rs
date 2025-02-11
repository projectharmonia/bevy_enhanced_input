use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn layering() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(first_binding)
        .add_observer(second_binding);

    let entity = app.world_mut().spawn((First, Second)).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();

    let first = registry.context::<First>(entity);
    assert_eq!(first.action::<DummyAction>().state(), ActionState::Fired);

    let second = registry.context::<Second>(entity);
    assert_eq!(second.action::<DummyAction>().state(), ActionState::None);

    app.world_mut().entity_mut(entity).remove::<First>();

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let second = registry.context::<Second>(entity);
    assert_eq!(
        second.action::<DummyAction>().state(),
        ActionState::None,
        "action should still be consumed even after removal"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(DummyAction::KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let second = registry.context::<Second>(entity);
    assert_eq!(second.action::<DummyAction>().state(), ActionState::Fired);
}

#[test]
fn switching() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(first_binding)
        .add_observer(second_binding);

    let entity = app.world_mut().spawn(First).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<First>(entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::Fired);

    app.world_mut()
        .entity_mut(entity)
        .remove::<First>()
        .insert(Second);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let second = registry.context::<Second>(entity);
    assert_eq!(
        second.action::<DummyAction>().state(),
        ActionState::None,
        "action should still be consumed even after removal"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(DummyAction::KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let second = registry.context::<Second>(entity);
    assert_eq!(second.action::<DummyAction>().state(), ActionState::Fired);
}

fn first_binding(mut trigger: Trigger<Binding<First>>) {
    trigger.set_priority(1);
    trigger.bind::<DummyAction>().to(DummyAction::KEY);
}

fn second_binding(mut trigger: Trigger<Binding<Second>>) {
    trigger.bind::<DummyAction>().to(DummyAction::KEY);
}

#[derive(Debug, Component)]
struct First;

#[derive(Debug, Component)]
struct Second;

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct DummyAction;

impl DummyAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
