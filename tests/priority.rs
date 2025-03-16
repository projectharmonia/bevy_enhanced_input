use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn prioritization() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(first_binding)
        .add_observer(second_binding);

    let entity = app.world_mut().spawn((First, Second)).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();

    let first = registry.context::<First>(entity);
    assert_eq!(first.action::<FirstConsume>().state(), ActionState::Fired);
    assert_eq!(
        first.action::<FirstPassthrough>().state(),
        ActionState::Fired
    );

    let second = registry.context::<Second>(entity);
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

fn first_binding(mut trigger: Trigger<Binding<First>>) {
    trigger.set_priority(1);
    trigger.bind::<FirstConsume>().to(CONSUME_KEY);
    trigger.bind::<FirstPassthrough>().to(PASSTHROUGH_KEY);
}

fn second_binding(mut trigger: Trigger<Binding<Second>>) {
    trigger.bind::<SecondConsume>().to(CONSUME_KEY);
    trigger.bind::<SecondPassthrough>().to(PASSTHROUGH_KEY);
}

#[derive(Debug, Component)]
struct First;

#[derive(Debug, Component)]
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
