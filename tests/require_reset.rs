use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn layering() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(bind_first)
        .add_observer(bind_second)
        .finish();

    let entity = app
        .world_mut()
        .spawn((Actions::<First>::default(), Actions::<Second>::default()))
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(TestAction::KEY);

    app.update();

    let first = app.world().get::<Actions<First>>(entity).unwrap();
    assert_eq!(first.state::<TestAction>()?, ActionState::Fired);

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(second.state::<TestAction>()?, ActionState::None);

    app.world_mut()
        .entity_mut(entity)
        .remove::<Actions<First>>();

    app.update();

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(
        second.state::<TestAction>()?,
        ActionState::None,
        "action should still be consumed even after removal"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(TestAction::KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(TestAction::KEY);

    app.update();

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(second.state::<TestAction>()?, ActionState::Fired);

    Ok(())
}

#[test]
fn switching() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(bind_first)
        .add_observer(bind_second)
        .finish();

    let entity = app.world_mut().spawn(Actions::<First>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(TestAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<First>>(entity).unwrap();
    assert_eq!(actions.state::<TestAction>()?, ActionState::Fired);

    app.world_mut()
        .entity_mut(entity)
        .remove::<Actions<First>>()
        .insert(Actions::<Second>::default());

    app.update();

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(
        second.state::<TestAction>()?,
        ActionState::None,
        "action should still be consumed even after removal"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(TestAction::KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(TestAction::KEY);

    app.update();

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(second.state::<TestAction>()?, ActionState::Fired);

    Ok(())
}

fn bind_first(trigger: Trigger<Bind<First>>, mut actions: Query<&mut Actions<First>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<TestAction>().to(TestAction::KEY);
}

fn bind_second(trigger: Trigger<Bind<Second>>, mut actions: Query<&mut Actions<Second>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<TestAction>().to(TestAction::KEY);
}

#[derive(InputContext)]
#[input_context(priority = 1)]
struct First;

#[derive(InputContext)]
struct Second;

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct TestAction;

impl TestAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
