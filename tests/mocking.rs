use core::time::Duration;

use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn updates() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .finish();

    let mut actions = Actions::<Test>::default();
    actions.mock_once::<TestAction>(ActionState::Fired, true);
    let entity = app.world_mut().spawn(actions).id();

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), true.into());
    assert_eq!(action.state(), ActionState::Fired);
    assert_eq!(action.events(), ActionEvents::FIRED | ActionEvents::STARTED);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), false.into());
    assert_eq!(action.state(), ActionState::None);
    assert_eq!(action.events(), ActionEvents::COMPLETED);
}

#[test]
fn duration() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(1)))
        .add_input_context::<Test>()
        .finish();

    // Update once to get a non-zero delta-time.
    app.update();

    let mut actions = Actions::<Test>::default();
    actions.mock::<TestAction>(ActionState::Fired, true, Duration::from_millis(2));
    let entity = app.world_mut().spawn(actions).id();

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), true.into());
    assert_eq!(action.state(), ActionState::Fired);
    assert_eq!(action.events(), ActionEvents::FIRED | ActionEvents::STARTED);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), true.into());
    assert_eq!(action.state(), ActionState::Fired);
    assert_eq!(action.events(), ActionEvents::FIRED);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), false.into());
    assert_eq!(action.state(), ActionState::None);
    assert_eq!(action.events(), ActionEvents::COMPLETED);
}

#[test]
fn manual() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .finish();

    let mut actions = Actions::<Test>::default();
    actions.mock::<TestAction>(ActionState::Fired, true, MockSpan::Manual);
    let entity = app.world_mut().spawn(actions).id();

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), true.into());
    assert_eq!(action.state(), ActionState::Fired);
    assert_eq!(action.events(), ActionEvents::FIRED | ActionEvents::STARTED);

    app.update();

    let mut actions = app.world_mut().get_mut::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), true.into());
    assert_eq!(action.state(), ActionState::Fired);
    assert_eq!(action.events(), ActionEvents::FIRED);

    actions.clear_mock::<TestAction>();

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    let action = actions.get::<TestAction>().unwrap();
    assert_eq!(action.value(), false.into());
    assert_eq!(action.state(), ActionState::None);
    assert_eq!(action.events(), ActionEvents::COMPLETED);
}

#[derive(InputContext)]
struct Test;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct TestAction;
