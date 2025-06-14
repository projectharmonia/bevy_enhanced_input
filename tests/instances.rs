use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(bind)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut().entity_mut(entity).remove::<Actions<Test>>();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(TestAction::KEY);

    app.world_mut()
        .add_observer(|_: Trigger<Fired<TestAction>>| {
            panic!("action shouldn't trigger");
        });

    app.update();
}

#[test]
fn rebuild() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(bind)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .entity_mut(entity)
        .insert(Actions::<Test>::default());

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert!(actions.get::<TestAction>().is_ok());
}

#[test]
fn rebuild_all() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(bind)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(TestAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.state::<TestAction>()?, ActionState::Fired);

    app.world_mut().trigger(RebuildBindings);
    app.world_mut().flush();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(
        actions.state::<TestAction>()?,
        ActionState::None,
        "state should reset on rebuild"
    );

    Ok(())
}

fn bind(trigger: Trigger<Bind<Test>>, mut actions: Query<&mut Actions<Test>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<TestAction>().to(TestAction::KEY);
}

#[derive(InputContext)]
struct Test;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct TestAction;

impl TestAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
