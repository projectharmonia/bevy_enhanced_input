use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn layering() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins))
        .add_input_context::<First>()
        .add_input_context::<Second>();

    let entity = app.world_mut().spawn((First, Second)).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();

    let first = instances.context::<First>(entity);
    assert_eq!(first.action::<DummyAction>().state(), ActionState::Fired);

    let second = instances.context::<Second>(entity);
    assert_eq!(second.action::<DummyAction>().state(), ActionState::None);

    app.world_mut().entity_mut(entity).remove::<First>();

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let second = instances.context::<Second>(entity);
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

    let instances = app.world().resource::<ContextInstances>();
    let second = instances.context::<Second>(entity);
    assert_eq!(second.action::<DummyAction>().state(), ActionState::Fired);
}

#[test]
fn switching() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins))
        .add_input_context::<First>()
        .add_input_context::<Second>();

    let entity = app.world_mut().spawn(First).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.context::<First>(entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::Fired);

    app.world_mut()
        .entity_mut(entity)
        .remove::<First>()
        .insert(Second);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let second = instances.context::<Second>(entity);
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

    let instances = app.world().resource::<ContextInstances>();
    let second = instances.context::<Second>(entity);
    assert_eq!(second.action::<DummyAction>().state(), ActionState::Fired);
}

#[derive(Debug, Component)]
struct First;

impl InputContext for First {
    const PRIORITY: isize = Second::PRIORITY + 1;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<DummyAction>().to(DummyAction::KEY);
        ctx
    }
}

#[derive(Debug, Component)]
struct Second;

impl InputContext for Second {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<DummyAction>().to(DummyAction::KEY);
        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct DummyAction;

impl DummyAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
