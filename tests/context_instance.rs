use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn context_removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(DummyAction::KEY);

    app.update();

    app.world_mut().entity_mut(entity).remove::<DummyContext>();

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    assert!(instances.get::<DummyContext>(entity).is_none());
}

#[test]
fn context_rebuild() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(DummyAction::KEY);

    app.update();

    app.world_mut().trigger(RebuildInputContexts);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::None);
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
