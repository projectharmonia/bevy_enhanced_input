use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn prioritization() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .add_observer(bind::<First>)
        .add_observer(bind::<Second>)
        .finish();

    let entity = app
        .world_mut()
        .spawn((Actions::<First>::default(), Actions::<Second>::default()))
        .id();

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(Consume::KEY);
    keys.press(Passthrough::KEY);

    app.update();

    let first = app.world().get::<Actions<First>>(entity).unwrap();
    assert_eq!(first.state::<Consume>()?, ActionState::Fired);
    assert_eq!(first.state::<Passthrough>()?, ActionState::Fired);

    let second = app.world().get::<Actions<Second>>(entity).unwrap();
    assert_eq!(
        second.state::<Consume>()?,
        ActionState::None,
        "action should be consumed by component input with a higher priority"
    );
    assert_eq!(
        second.state::<Passthrough>()?,
        ActionState::Fired,
        "actions that doesn't consume inputs should still be triggered"
    );

    Ok(())
}

fn bind<C: InputContext>(trigger: Trigger<Bind<C>>, mut actions: Query<&mut Actions<C>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Consume>().to(Consume::KEY);
    actions.bind::<Passthrough>().to(Passthrough::KEY);
}

#[derive(InputContext)]
#[input_context(priority = 1)]
struct First;

#[derive(InputContext)]
struct Second;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct Consume;

impl Consume {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct Passthrough;

impl Passthrough {
    const KEY: KeyCode = KeyCode::KeyB;
}
