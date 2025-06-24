use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn same_schedule() -> Result<()> {
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

#[test]
fn different_schedules() -> Result<()> {
    let time_step = Time::<Fixed>::default().timestep() / 2;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        .add_input_context::<First>()
        .add_input_context::<FixedSchedule>()
        .add_observer(bind::<First>)
        .add_observer(bind::<FixedSchedule>)
        .finish();

    let entity = app
        .world_mut()
        .spawn((
            Actions::<First>::default(),
            Actions::<FixedSchedule>::default(),
        ))
        .id();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(Consume::KEY);
    keys.press(Passthrough::KEY);

    for frame in 0..2 {
        app.update();

        let actions = app.world().get::<Actions<First>>(entity).unwrap();
        assert_eq!(actions.state::<Consume>()?, ActionState::Fired);
        assert_eq!(actions.state::<Passthrough>()?, ActionState::Fired);

        let fixed_actions = app.world().get::<Actions<FixedSchedule>>(entity).unwrap();
        assert_eq!(fixed_actions.state::<Consume>()?, ActionState::None);
        assert_eq!(
            fixed_actions.state::<Passthrough>()?,
            ActionState::None,
            "shouldn't fire on frame {frame} because the schedule hasn't run yet"
        );
    }

    for frame in 2..4 {
        app.update();

        let actions = app.world().get::<Actions<First>>(entity).unwrap();
        assert_eq!(actions.state::<Consume>()?, ActionState::Fired);
        assert_eq!(actions.state::<Passthrough>()?, ActionState::Fired);

        let fixed_actions = app.world().get::<Actions<FixedSchedule>>(entity).unwrap();
        assert_eq!(
            fixed_actions.state::<Consume>()?,
            ActionState::None,
            "shouldn't fire on frame {frame} because of the schedule evaluation order"
        );
        assert_eq!(fixed_actions.state::<Passthrough>()?, ActionState::Fired);
    }

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

#[derive(InputContext)]
#[input_context(schedule = FixedPreUpdate, priority = 3)]
struct FixedSchedule;

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
