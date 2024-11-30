use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn exclusive() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Exclusive>();

    let entity1 = app.world_mut().spawn(Exclusive).id();
    let entity2 = app.world_mut().spawn(Exclusive).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ExclusiveConsume::KEY);
    keys.press(ExclusivePassthrough::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();

    let ctx = instances.get::<Exclusive>(entity1).unwrap();

    let action = ctx.action::<ExclusiveConsume>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let action = ctx.action::<ExclusivePassthrough>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let ctx = instances.get::<Exclusive>(entity2).unwrap();

    let action = ctx.action::<ExclusiveConsume>().unwrap();
    assert_eq!(
        action.state(),
        ActionState::None,
        "only first entity with the same mappings that consume inputs should receive them"
    );

    let action = ctx.action::<ExclusivePassthrough>().unwrap();
    assert_eq!(
        action.state(),
        ActionState::Fired,
        "actions that doesn't consume inputs should still fire"
    );
}

#[test]
fn shared() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Shared>();

    let entity1 = app.world_mut().spawn(Shared).id();
    let entity2 = app.world_mut().spawn(Shared).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(SharedConsume::KEY);
    keys.press(SharedPassthrough::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();

    let ctx = instances.get::<Shared>(entity1).unwrap();

    let action = ctx.action::<SharedConsume>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let action = ctx.action::<SharedPassthrough>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let ctx = instances.get::<Shared>(entity2).unwrap();

    let action = ctx.action::<SharedConsume>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let action = ctx.action::<SharedPassthrough>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);
}

#[test]
fn context_removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Exclusive>()
        .add_input_context::<Shared>();

    let entity = app.world_mut().spawn((Exclusive, Shared)).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ExclusiveConsume::KEY);
    keys.press(SharedConsume::KEY);

    app.update();

    app.world_mut()
        .entity_mut(entity)
        .remove::<Exclusive>()
        .remove::<Shared>();

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    assert!(instances.get::<Exclusive>(entity).is_none());
    assert!(instances.get::<Shared>(entity).is_none());
}

#[test]
fn context_rebuild() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Exclusive>()
        .add_input_context::<Shared>();

    let entity = app.world_mut().spawn((Exclusive, Shared)).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ExclusiveConsume::KEY);
    keys.press(SharedConsume::KEY);

    app.update();

    app.world_mut().trigger(RebuildInputContexts);

    app.update();

    let instances = app.world().resource::<ContextInstances>();

    let ctx = instances.get::<Exclusive>(entity).unwrap();
    let action = ctx.action::<ExclusiveConsume>().unwrap();
    assert_eq!(action.state(), ActionState::None);

    let ctx = instances.get::<Shared>(entity).unwrap();
    let action = ctx.action::<SharedConsume>().unwrap();
    assert_eq!(action.state(), ActionState::None);
}

#[derive(Debug, Component)]
struct Exclusive;

impl InputContext for Exclusive {
    const MODE: ContextMode = ContextMode::Exclusive;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<ExclusiveConsume>().to(ExclusiveConsume::KEY);
        ctx.bind::<ExclusivePassthrough>()
            .to(ExclusivePassthrough::KEY);
        ctx
    }
}

#[derive(Debug, Component)]
struct Shared;

impl InputContext for Shared {
    const MODE: ContextMode = ContextMode::Shared;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<SharedConsume>().to(SharedConsume::KEY);
        ctx.bind::<SharedPassthrough>().to(SharedPassthrough::KEY);
        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct ExclusiveConsume;

impl ExclusiveConsume {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct ExclusivePassthrough;

impl ExclusivePassthrough {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct SharedConsume;

impl SharedConsume {
    const KEY: KeyCode = KeyCode::KeyC;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct SharedPassthrough;

impl SharedPassthrough {
    const KEY: KeyCode = KeyCode::KeyD;
}
