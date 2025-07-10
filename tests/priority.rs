use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn same_schedule() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    app.world_mut().spawn((
        First,
        actions!(First[
            (
                Action::<FirstConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<FirstPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
        Second,
        actions!(Second[
            (
                Action::<SecondConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<SecondPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
    ));

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    app.update();

    let mut first_consume = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<FirstConsume>>>();

    let first_consume_state = *first_consume.single(app.world()).unwrap();
    assert_eq!(first_consume_state, ActionState::Fired);

    let mut first_passthrough = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<FirstPassthrough>>>();

    let first_passthrough_state = *first_passthrough.single(app.world()).unwrap();
    assert_eq!(first_passthrough_state, ActionState::Fired);

    let mut second_consume = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<SecondConsume>>>();

    let second_consume_state = *second_consume.single(app.world()).unwrap();
    assert_eq!(
        second_consume_state,
        ActionState::None,
        "action should be consumed by component input with a higher priority"
    );

    let mut second_passthrough = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<SecondPassthrough>>>();

    let second_passthrough_state = *second_passthrough.single(app.world()).unwrap();
    assert_eq!(
        second_passthrough_state,
        ActionState::Fired,
        "actions that doesn't consume inputs should still be triggered"
    );
}

#[test]
fn different_schedules() {
    let time_step = Time::<Fixed>::default().timestep() / 2;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        .add_input_context::<First>()
        .add_input_context::<FixedSecond>()
        .finish();

    app.world_mut().spawn((
        First,
        actions!(First[
            (
                Action::<FirstConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<FirstPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
        FixedSecond,
        actions!(FixedSecond[
            (
                Action::<SecondConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<SecondPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
    ));

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    let mut first_consume = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<FirstConsume>>>();
    let mut first_passthrough = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<FirstPassthrough>>>();
    let mut second_consume = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<SecondConsume>>>();
    let mut second_passthrough = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<SecondPassthrough>>>();

    for frame in 0..2 {
        app.update();

        let first_consume_state = *first_consume.single(app.world()).unwrap();
        assert_eq!(first_consume_state, ActionState::Fired);

        let first_passthrough_state = *first_passthrough.single(app.world()).unwrap();
        assert_eq!(first_passthrough_state, ActionState::Fired);

        let second_consume_state = *second_consume.single(app.world()).unwrap();
        assert_eq!(second_consume_state, ActionState::None);

        let second_passthrough_state = *second_passthrough.single(app.world()).unwrap();
        assert_eq!(
            second_passthrough_state,
            ActionState::None,
            "shouldn't fire on frame {frame} because the schedule hasn't run yet"
        );
    }

    for frame in 2..4 {
        app.update();

        let first_consume_state = *first_consume.single(app.world()).unwrap();
        assert_eq!(first_consume_state, ActionState::Fired);

        let first_passthrough_state = *first_passthrough.single(app.world()).unwrap();
        assert_eq!(first_passthrough_state, ActionState::Fired);

        let second_consume_state = *second_consume.single(app.world()).unwrap();
        assert_eq!(
            second_consume_state,
            ActionState::None,
            "shouldn't fire on frame {frame} because of the schedule evaluation order"
        );

        let second_passthrough_state = *second_passthrough.single(app.world()).unwrap();
        assert_eq!(second_passthrough_state, ActionState::Fired);
    }
}

#[derive(Component, InputContext)]
#[input_context(priority = 1)]
struct First;

#[derive(Component, InputContext)]
struct Second;

#[derive(Component, InputContext)]
#[input_context(schedule = FixedPreUpdate, priority = 3)]
struct FixedSecond;

/// A key used by all consume actions.
const CONSUME_KEY: KeyCode = KeyCode::KeyA;

/// A key used by all consume actions.
const PASSTHROUGH_KEY: KeyCode = KeyCode::KeyB;

#[derive(InputAction)]
#[input_action(output = bool)]
struct FirstConsume;

#[derive(InputAction)]
#[input_action(output = bool)]
struct FirstPassthrough;

#[derive(InputAction)]
#[input_action(output = bool)]
struct SecondConsume;

#[derive(InputAction)]
#[input_action(output = bool)]
struct SecondPassthrough;
