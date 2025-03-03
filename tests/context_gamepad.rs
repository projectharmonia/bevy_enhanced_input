use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn any() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<AnyGamepad>()
        .add_observer(any_gamepad_binding);

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    let context_entity = app.world_mut().spawn(AnyGamepad).id();

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<AnyGamepad>(context_entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().release(DummyAction::BUTTON);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<AnyGamepad>(context_entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::Fired);
}

#[test]
fn by_id() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<SingleGamepad>()
        .add_observer(single_gamepad_binding);

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    let context_entity = app.world_mut().spawn(SingleGamepad(gamepad_entity1)).id();

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<SingleGamepad>(context_entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().release(DummyAction::BUTTON);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let registry = app.world().resource::<InputContextRegistry>();
    let ctx = registry.context::<SingleGamepad>(context_entity);
    assert_eq!(ctx.action::<DummyAction>().state(), ActionState::None);
}

fn any_gamepad_binding(mut trigger: Trigger<Binding<AnyGamepad>>) {
    trigger.bind::<DummyAction>().to(DummyAction::BUTTON);
}

fn single_gamepad_binding(
    mut trigger: Trigger<Binding<SingleGamepad>>,
    gamepads: Query<&SingleGamepad>,
) {
    let gamepad_entity = **gamepads.get(trigger.entity()).unwrap();
    trigger.set_gamepad(gamepad_entity);
    trigger.bind::<DummyAction>().to(DummyAction::BUTTON);
}

#[derive(Debug, Component)]
struct AnyGamepad;

#[derive(Debug, Component, Deref)]
struct SingleGamepad(Entity);

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const BUTTON: GamepadButton = GamepadButton::South;
}
