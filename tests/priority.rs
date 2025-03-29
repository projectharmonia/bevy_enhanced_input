use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn prioritization() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_actions_marker::<First>()
        .add_actions_marker::<Second>()
        .add_observer(first_binding)
        .add_observer(second_binding)
        .finish();

    let entity = app
        .world_mut()
        .spawn((Actions::<First>::default(), Actions::<Second>::default()))
        .id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    app.update();

    let first = app.world().get::<Actions<First>>(entity).unwrap();
    assert_eq!(first.action::<FirstConsume>().state(), ActionState::Fired);
    assert_eq!(
        first.action::<FirstPassthrough>().state(),
        ActionState::Fired
    );

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(
        second.action::<SecondConsume>().state(),
        ActionState::None,
        "action should be consumed by component input with a higher priority"
    );
    assert_eq!(
        second.action::<SecondPassthrough>().state(),
        ActionState::Fired,
        "actions that doesn't consume inputs should still be triggered"
    );
}

fn first_binding(trigger: Trigger<Binding<First>>, mut actions: Query<&mut Actions<First>>) {
    let mut actions = actions.get_mut(trigger.entity()).unwrap();
    actions.bind::<FirstConsume>().to(CONSUME_KEY);
    actions.bind::<FirstPassthrough>().to(PASSTHROUGH_KEY);
}

fn second_binding(trigger: Trigger<Binding<Second>>, mut actions: Query<&mut Actions<Second>>) {
    let mut actions = actions.get_mut(trigger.entity()).unwrap();
    actions.bind::<SecondConsume>().to(CONSUME_KEY);
    actions.bind::<SecondPassthrough>().to(PASSTHROUGH_KEY);
}

#[derive(ActionsMarker)]
#[actions_marker(priority = 1)]
struct First;

#[derive(ActionsMarker)]
struct Second;

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
