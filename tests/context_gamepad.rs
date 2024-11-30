use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn any() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<AnyGamepad>();

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    let context_entity = app.world_mut().spawn(AnyGamepad).id();

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<AnyGamepad>(context_entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().release(DummyAction::BUTTON);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<AnyGamepad>(context_entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);
}

#[test]
fn by_id() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<SingleGamepad>();

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    let context_entity = app.world_mut().spawn(SingleGamepad(gamepad_entity1)).id();

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<SingleGamepad>(context_entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.digital_mut().release(DummyAction::BUTTON);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.digital_mut().press(DummyAction::BUTTON);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<SingleGamepad>(context_entity).unwrap();
    let action = ctx.action::<DummyAction>().unwrap();
    assert_eq!(action.state(), ActionState::None);
}

#[derive(Debug, Component)]
struct AnyGamepad;

impl InputContext for AnyGamepad {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();
        ctx.bind::<DummyAction>().to(DummyAction::BUTTON);
        ctx
    }
}

#[derive(Debug, Component, Deref)]
struct SingleGamepad(Entity);

impl InputContext for SingleGamepad {
    fn context_instance(world: &World, entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        let gamepad_entity = **world.get::<Self>(entity).unwrap();
        ctx.set_gamepad(gamepad_entity);
        ctx.bind::<DummyAction>().to(DummyAction::BUTTON);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct DummyAction;

impl DummyAction {
    const BUTTON: GamepadButton = GamepadButton::South;
}
