use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let context = app
        .world_mut()
        .spawn((
            TestContext,
            actions!(TestContext[(Action::<Test>::new(), bindings![Test::KEY])]),
        ))
        .id();

    app.update();

    app.world_mut().entity_mut(context).remove::<TestContext>();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    app.world_mut().add_observer(|_: Trigger<Fired<Test>>| {
        panic!("action shouldn't trigger");
    });

    app.update();
}

#[derive(Component, InputContext)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;

impl Test {
    const KEY: KeyCode = KeyCode::KeyA;
}
