use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn explicit() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_observer(bind)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Player>::default()).id();

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Explicit>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Explicit::KEY);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Explicit>()?;
    assert_eq!(action.value, true.into());
    assert_eq!(action.state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Explicit::KEY);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Explicit>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    Ok(())
}

#[test]
fn implicit() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_observer(bind)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Player>::default()).id();

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Implicit>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ReleaseAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, true.into());
    assert_eq!(action.state, ActionState::Ongoing);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Implicit>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(ReleaseAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::Fired);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Implicit>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::Fired);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Implicit>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    Ok(())
}

#[test]
fn blocker() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_observer(bind)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Player>::default()).id();

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Blocker>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(ReleaseAction::KEY);
    keys.press(Blocker::KEY);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, true.into());
    assert_eq!(action.state, ActionState::Ongoing);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Blocker>()?;
    assert_eq!(action.value, true.into());
    assert_eq!(action.state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(ReleaseAction::KEY);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::Fired);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Blocker>()?;
    assert_eq!(action.value, true.into());
    assert_eq!(action.state, ActionState::None);

    app.update();

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<ReleaseAction>()?;
    assert_eq!(action.value, false.into());
    assert_eq!(action.state, ActionState::None);

    let actions = app.world().get::<Actions<Player>>(entity).unwrap();
    let action = actions.get::<Blocker>()?;
    assert_eq!(action.value, true.into());
    assert_eq!(action.state, ActionState::Fired);

    Ok(())
}

fn bind(trigger: Trigger<Bind<Player>>, mut actions: Query<&mut Actions<Player>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<ReleaseAction>()
        .to(ReleaseAction::KEY)
        .with_conditions(Release::default());
    actions
        .bind::<Explicit>()
        .with_conditions(Down::default())
        .to(Explicit::KEY);
    actions
        .bind::<Implicit>()
        .with_conditions(Chord::<ReleaseAction>::default());
    actions
        .bind::<Blocker>()
        .to(Blocker::KEY)
        .with_conditions(BlockBy::<ReleaseAction>::default());
}

#[derive(InputContext)]
struct Player;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ReleaseAction;

impl ReleaseAction {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Explicit;

impl Explicit {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Implicit;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Blocker;

impl Blocker {
    const KEY: KeyCode = KeyCode::KeyD;
}
