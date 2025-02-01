use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn prioritization() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugins))
        .add_input_context::<First>()
        .add_input_context::<Second>();

    let entity = app.world_mut().spawn((First, Second)).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();

    let first = instances.context::<First>(entity);
    assert_eq!(first.action::<FirstConsume>().state(), ActionState::Fired);
    assert_eq!(
        first.action::<FirstPassthrough>().state(),
        ActionState::Fired
    );

    let second = instances.context::<Second>(entity);
    assert_eq!(
        second.action::<SecondConsume>().state(),
        ActionState::None,
        "action should be consumed by context with a higher priority"
    );
    assert_eq!(
        second.action::<SecondPassthrough>().state(),
        ActionState::Fired,
        "actions that doesn't consume inputs should still be triggered"
    );
}

#[derive(Debug, Component)]
struct First;

impl InputContext for First {
    const PRIORITY: isize = Second::PRIORITY + 1;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<FirstConsume>().to(CONSUME_KEY);
        ctx.bind::<FirstPassthrough>().to(PASSTHROUGH_KEY);

        ctx
    }
}

#[derive(Debug, Component)]
struct Second;

impl InputContext for Second {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<SecondConsume>().to(CONSUME_KEY);
        ctx.bind::<SecondPassthrough>().to(PASSTHROUGH_KEY);

        ctx
    }
}

/// A key used by both [`FirstConsume`] and [`SecondConsume`] actions.
const CONSUME_KEY: KeyCode = KeyCode::KeyA;

/// A key used by both [`FirstPassthrough`] and [`SecondPassthrough`] actions.
const PASSTHROUGH_KEY: KeyCode = KeyCode::KeyB;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct FirstConsume;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct SecondConsume;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct FirstPassthrough;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct SecondPassthrough;
