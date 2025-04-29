use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn layering() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(first_binding)
        .add_observer(second_binding)
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
    assert_eq!(first.action::<TestAction>().state(), ActionState::Fired);

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(second.action::<TestAction>().state(), ActionState::None);

    app.world_mut()
        .entity_mut(entity)
        .remove::<Actions<First>>();

    app.update();

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(
        second.action::<TestAction>().state(),
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
    assert_eq!(second.action::<TestAction>().state(), ActionState::Fired);
}

#[test]
fn switching() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(first_binding)
        .add_observer(second_binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<First>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(TestAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<First>>(entity).unwrap();
    assert_eq!(actions.action::<TestAction>().state(), ActionState::Fired);

    app.world_mut()
        .entity_mut(entity)
        .remove::<Actions<First>>()
        .insert(Actions::<Second>::default());

    app.update();

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(
        second.action::<TestAction>().state(),
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
    assert_eq!(second.action::<TestAction>().state(), ActionState::Fired);
}

fn first_binding(trigger: Trigger<Binding<First>>, mut actions: Query<&mut Actions<First>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<TestAction>().to(TestAction::KEY);
}

fn second_binding(trigger: Trigger<Binding<Second>>, mut actions: Query<&mut Actions<Second>>) {
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
