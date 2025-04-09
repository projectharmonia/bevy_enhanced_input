use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn max_abs() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Dummy>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Dummy>::default()).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);

    app.update();

    let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
    assert_eq!(actions.action::<MaxAbs>().value(), Vec2::Y.into());
}

#[test]
fn cumulative() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Dummy>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Dummy>::default()).id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowDown);

    app.update();

    let actions = app.world().get::<Actions<Dummy>>(entity).unwrap();
    assert_eq!(
        actions.action::<Cumulative>().value(),
        Vec2::ZERO.into(),
        "up and down should cancel each other"
    );
}

fn binding(trigger: Trigger<Binding<Dummy>>, mut actions: Query<&mut Actions<Dummy>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<MaxAbs>().to(Cardinal::wasd_keys());
    actions.bind::<Cumulative>().to(Cardinal::arrow_keys());
}

#[derive(InputContext)]
struct Dummy;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, accumulation = MaxAbs)]
struct MaxAbs;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, accumulation = Cumulative)]
struct Cumulative;
