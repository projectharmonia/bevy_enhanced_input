use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn any() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<AnyGamepad>()
        .add_observer(bind_any_gamepad)
        .finish();

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    let context_entity = app.world_mut().spawn(Actions::<AnyGamepad>::default()).id();

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(TestAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<AnyGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.state::<TestAction>()?, ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(TestAction::BUTTON, 0.0);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.analog_mut().set(TestAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<AnyGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.state::<TestAction>()?, ActionState::Fired);

    Ok(())
}

#[test]
fn by_id() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<SingleGamepad>()
        .add_observer(bind_single_gamepad)
        .finish();

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    let context_entity = app
        .world_mut()
        .spawn((
            Actions::<SingleGamepad>::default(),
            SingleGamepad(gamepad_entity1),
        ))
        .id();

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(TestAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<SingleGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.state::<TestAction>()?, ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(TestAction::BUTTON, 0.0);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.analog_mut().set(TestAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<SingleGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.state::<TestAction>()?, ActionState::None);

    Ok(())
}

fn bind_any_gamepad(
    trigger: Trigger<Bind<AnyGamepad>>,
    mut actions: Query<&mut Actions<AnyGamepad>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<TestAction>().to(TestAction::BUTTON);
}

fn bind_single_gamepad(
    trigger: Trigger<Bind<SingleGamepad>>,
    mut actions: Query<(&mut Actions<SingleGamepad>, &mut SingleGamepad)>,
) {
    let (mut actions, gamepad) = actions.get_mut(trigger.target()).unwrap();
    actions.set_gamepad(**gamepad);
    actions.bind::<TestAction>().to(TestAction::BUTTON);
}

#[derive(InputContext)]
struct AnyGamepad;

#[derive(Component, Deref, InputContext)]
struct SingleGamepad(Entity);

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct TestAction;

impl TestAction {
    const BUTTON: GamepadButton = GamepadButton::South;
}
