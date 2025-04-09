use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn any() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<AnyGamepad>()
        .add_observer(any_gamepad_binding)
        .finish();

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    let context_entity = app.world_mut().spawn(Actions::<AnyGamepad>::default()).id();

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(DummyAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<AnyGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.action::<DummyAction>().state(), ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(DummyAction::BUTTON, 0.0);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.analog_mut().set(DummyAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<AnyGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.action::<DummyAction>().state(), ActionState::Fired);
}

#[test]
fn by_id() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<SingleGamepad>()
        .add_observer(single_gamepad_binding)
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
    gamepad1.analog_mut().set(DummyAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<SingleGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.action::<DummyAction>().state(), ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(DummyAction::BUTTON, 0.0);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.analog_mut().set(DummyAction::BUTTON, 1.0);

    app.update();

    let actions = app
        .world()
        .get::<Actions<SingleGamepad>>(context_entity)
        .unwrap();
    assert_eq!(actions.action::<DummyAction>().state(), ActionState::None);
}

fn any_gamepad_binding(
    trigger: Trigger<Binding<AnyGamepad>>,
    mut actions: Query<&mut Actions<AnyGamepad>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<DummyAction>().to(DummyAction::BUTTON);
}

fn single_gamepad_binding(
    trigger: Trigger<Binding<SingleGamepad>>,
    mut actions: Query<(&mut Actions<SingleGamepad>, &mut SingleGamepad)>,
) {
    let (mut actions, gamepad) = actions.get_mut(trigger.target()).unwrap();
    actions.set_gamepad(**gamepad);
    actions.bind::<DummyAction>().to(DummyAction::BUTTON);
}

#[derive(InputContext)]
struct AnyGamepad;

#[derive(Component, Deref, InputContext)]
struct SingleGamepad(Entity);

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const BUTTON: GamepadButton = GamepadButton::South;
}
