use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>()
        .add_observer(binding);

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    assert!(registry.get_context::<DummyContext>(entity).is_some());

    app.update();

    app.world_mut().entity_mut(entity).remove::<DummyContext>();

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    assert!(registry.get_context::<DummyContext>(entity).is_none());
}

#[test]
fn rebuild() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>()
        .add_observer(binding);

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    assert!(registry.get_context::<DummyContext>(entity).is_some());

    app.update();

    app.world_mut().entity_mut(entity).insert(DummyContext);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    assert!(registry.get_context::<DummyContext>(entity).is_some());
}

#[test]
fn rebuild_all() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>()
        .add_observer(binding);

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<DummyContext>(entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::Fired);

    app.world_mut().trigger(RebuildBindings);
    app.world_mut().flush();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<DummyContext>(entity);
    assert_eq!(
        ctx.action::<DummyAction>().state(),
        ActionState::None,
        "state should reset on rebuild"
    );
}

fn binding(mut trigger: Trigger<Binding<DummyContext>>) {
    trigger.bind::<DummyAction>().to(DummyAction::KEY);
}

#[derive(Debug, Component)]
struct DummyContext;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
