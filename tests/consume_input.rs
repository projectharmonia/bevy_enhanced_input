use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn consume() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(consume_only_binding)
        .finish();

    let entity1 = app.world_mut().spawn(Actions::<Test>::default()).id();
    let entity2 = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let entity1_ctx = app.world().get::<Actions<Test>>(entity1).unwrap();
    assert_eq!(entity1_ctx.state::<Consume>()?, ActionState::Fired);

    let entity2_ctx = app.world().get::<Actions<Test>>(entity2).unwrap();
    assert_eq!(
        entity2_ctx.state::<Consume>()?,
        ActionState::None,
        "only first entity with the same mappings that consume inputs should receive them"
    );

    Ok(())
}

#[test]
fn passthrough() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(passthrough_only_binding)
        .finish();

    let entity1 = app.world_mut().spawn(Actions::<Test>::default()).id();
    let entity2 = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let entity1_ctx = app.world().get::<Actions<Test>>(entity1).unwrap();
    assert_eq!(entity1_ctx.state::<Passthrough>()?, ActionState::Fired);

    let entity2_ctx = app.world().get::<Actions<Test>>(entity2).unwrap();
    assert_eq!(
        entity2_ctx.state::<Passthrough>()?,
        ActionState::Fired,
        "actions that doesn't consume inputs should still fire"
    );

    Ok(())
}

#[test]
fn consume_then_passthrough() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(consume_then_passthrough_binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.state::<Consume>()?, ActionState::Fired);
    assert_eq!(
        actions.state::<Passthrough>()?,
        ActionState::None,
        "action should be consumed"
    );

    Ok(())
}

#[test]
fn passthrough_then_consume() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(passthrough_then_consume_binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.state::<Consume>()?, ActionState::Fired);
    assert_eq!(actions.state::<Passthrough>()?, ActionState::Fired);

    Ok(())
}

#[test]
fn modifiers() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(modifiers_binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.state::<NoModifiers>()?, ActionState::Fired);
    assert_eq!(actions.state::<WithModifiers>()?, ActionState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ControlLeft);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.state::<NoModifiers>()?, ActionState::None);
    assert_eq!(actions.state::<WithModifiers>()?, ActionState::Fired);

    Ok(())
}

fn consume_only_binding(trigger: Trigger<Binding<Test>>, mut actions: Query<&mut Actions<Test>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Consume>().to(KEY);
}

fn passthrough_only_binding(
    trigger: Trigger<Binding<Test>>,
    mut actions: Query<&mut Actions<Test>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Passthrough>().to(KEY);
}

fn consume_then_passthrough_binding(
    trigger: Trigger<Binding<Test>>,
    mut actions: Query<&mut Actions<Test>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Consume>().to(KEY);
    actions.bind::<Passthrough>().to(KEY);
}

fn passthrough_then_consume_binding(
    trigger: Trigger<Binding<Test>>,
    mut actions: Query<&mut Actions<Test>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Passthrough>().to(KEY);
    actions.bind::<Consume>().to(KEY);
}

fn modifiers_binding(trigger: Trigger<Binding<Test>>, mut actions: Query<&mut Actions<Test>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<NoModifiers>().to(KEY);
    actions
        .bind::<WithModifiers>()
        .to(KEY.with_mod_keys(WithModifiers::MOD));
}

#[derive(InputContext)]
struct Test;

/// A key used by all actions.
const KEY: KeyCode = KeyCode::KeyA;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = true)]
struct Consume;

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct Passthrough;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct NoModifiers;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct WithModifiers;

impl WithModifiers {
    const MOD: ModKeys = ModKeys::CONTROL;
}
