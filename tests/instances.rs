use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Dummy>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Dummy>::default()).id();

    app.update();

    app.world_mut()
        .entity_mut(entity)
        .remove::<Actions<Dummy>>();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.world_mut()
        .add_observer(|_: Trigger<Fired<DummyAction>>| {
            panic!("action shouldn't trigger");
        });

    app.update();
}

#[test]
fn rebuild() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Dummy>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Dummy>::default()).id();

    app.update();

    app.world_mut()
        .entity_mut(entity)
        .insert(Actions::<Dummy>::default());

    app.update();

    let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
    assert!(actions.get_action::<DummyAction>().is_some());
}

#[test]
fn rebuild_all() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Dummy>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Dummy>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
    assert_eq!(actions.action::<DummyAction>().state(), ActionState::Fired);

    app.world_mut().trigger(RebuildBindings);
    app.world_mut().flush();

    let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
    assert_eq!(
        actions.action::<DummyAction>().state(),
        ActionState::None,
        "state should reset on rebuild"
    );
}

fn binding(trigger: Trigger<Binding<Dummy>>, mut actions: Query<&mut Actions<Dummy>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<DummyAction>().to(DummyAction::KEY);
}

#[derive(InputContext)]
struct Dummy;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
