use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::prelude::*;

#[test]
fn once_in_two_frames() {
    let time_step = Time::<Fixed>::default().timestep() / 2;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        .add_input_context::<Dummy>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Dummy>::default()).id();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    for frame in 0..2 {
        app.update();

        let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
        assert!(
            actions.action::<DummyAction>().events().is_empty(),
            "shouldn't fire on frame {frame}"
        );
    }

    for frame in 2..4 {
        app.update();

        let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
        let action = actions.action::<DummyAction>();
        assert_eq!(
            action.events(),
            ActionEvents::STARTED | ActionEvents::FIRED,
            "should maintain start-firing on frame {frame}"
        );
    }
}

#[test]
fn twice_in_one_frame() {
    let time_step = Time::<Fixed>::default().timestep() * 2;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        .add_input_context::<Dummy>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Dummy>::default()).id();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(DummyAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
    assert!(
        actions.action::<DummyAction>().events().is_empty(),
        "`FixedMain` should never run on the first frame"
    );

    app.update();

    let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
    let action = actions.action::<DummyAction>();
    assert_eq!(
        action.events(),
        ActionEvents::FIRED,
        "should run twice, so it shouldn't be started on the second run"
    );
}

fn binding(trigger: Trigger<Binding<Dummy>>, mut actions: Query<&mut Actions<Dummy>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<DummyAction>().to(DummyAction::KEY);
}

#[derive(InputContext)]
#[input_context(schedule = FixedPreUpdate)]
struct Dummy;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const KEY: KeyCode = KeyCode::KeyA;
}
