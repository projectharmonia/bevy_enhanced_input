use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn layering() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    app.world_mut().spawn((
        First,
        actions!(
            First[(
                Action::<OnFirst>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KEY]
            )]
        ),
        Second,
        actions!(Second[(Action::<OnSecond>::new(), bindings![KEY])]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut first_actions = app
        .world_mut()
        .query_filtered::<(Entity, &ActionState), With<Action<OnFirst>>>();

    let (first_action, &first_state) = first_actions.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    let mut second_actions = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<OnSecond>>>();

    let second_state = *second_actions.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::None);

    app.world_mut().entity_mut(first_action).despawn();

    app.update();

    let second_state = *second_actions.single(app.world()).unwrap();
    assert_eq!(
        second_state,
        ActionState::None,
        "action should still be consumed even after removal"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let second_state = *second_actions.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::Fired);
}

#[test]
fn switching() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    let context = app
        .world_mut()
        .spawn((
            First,
            actions!(
                First[(
                    Action::<OnFirst>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
                    bindings![KEY]
                )]
            ),
        ))
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut actions = app.world_mut().query::<&ActionState>();

    let first_state = *actions.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    app.world_mut()
        .entity_mut(context)
        .remove::<First>()
        .despawn_related::<Actions<First>>()
        .insert((
            Second,
            actions!(Second[(Action::<OnSecond>::new(), bindings![KEY])]),
        ));

    app.update();

    let second_state = *actions.single(app.world()).unwrap();
    assert_eq!(
        second_state,
        ActionState::None,
        "action should still be consumed even after removal"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let second_state = *actions.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::Fired);
}

#[derive(Component, InputContext)]
#[input_context(priority = 1)]
struct First;

#[derive(Component, InputContext)]
struct Second;

/// A key used by all actions.
const KEY: KeyCode = KeyCode::KeyA;

#[derive(InputAction)]
#[action_output(bool)]
struct OnFirst;

#[derive(InputAction)]
#[action_output(bool)]
struct OnSecond;
