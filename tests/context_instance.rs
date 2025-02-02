use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    assert!(instances.get_context::<DummyContext>(entity).is_some());

    app.update();

    app.world_mut().entity_mut(entity).remove::<DummyContext>();

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    assert!(instances.get_context::<DummyContext>(entity).is_none());
}

#[test]
fn rebuild() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.context::<DummyContext>(entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::Fired);

    app.world_mut().trigger(RebuildInputContexts);

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.context::<DummyContext>(entity);
    assert_eq!(
        ctx.action::<DummyAction>().state(),
        ActionState::None,
        "state should reset on rebuild"
    );
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<DummyAction>().to(DummyAction::KEY);
        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
