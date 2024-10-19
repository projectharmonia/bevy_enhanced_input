mod action_recorder;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

use action_recorder::{ActionRecorderPlugin, AppTriggeredExt, RecordedActions};

#[test]
fn input_level() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<InputLevel>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(InputLevel::KEY1);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<InputLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, (Vec2::Y * 2.0).into());

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(InputLevel::KEY1);
    keys.press(InputLevel::KEY2);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<InputLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, Vec2::NEG_Y.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(InputLevel::KEY1);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<InputLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, Vec2::Y.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<InputLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(
        event.state,
        ActionState::None,
        "if a required condition fails, it should override regular conditions"
    );
}

#[test]
fn action_level() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<ActionLevel>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ActionLevel::KEY1);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ActionLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, (Vec2::NEG_Y * 2.0).into());

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(ActionLevel::KEY1);
    keys.press(ActionLevel::KEY2);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ActionLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, (Vec2::NEG_Y * 2.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ActionLevel::KEY1);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ActionLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, (Vec2::NEG_Y * 4.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<ActionLevel>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(
        event.state,
        ActionState::None,
        "if a required condition fails, it should override regular conditions"
    );
}

#[test]
fn both_levels() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        EnhancedInputPlugin,
        ActionRecorderPlugin,
    ))
    .add_input_context::<DummyContext>()
    .record_action::<BothLevels>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(BothLevels::KEY1);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<BothLevels>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, (Vec2::Y * 2.0).into());

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(BothLevels::KEY1);
    keys.press(BothLevels::KEY2);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<BothLevels>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, Vec2::NEG_Y.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(BothLevels::KEY1);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<BothLevels>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(event.value, Vec2::Y.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let recorded = app.world().resource::<RecordedActions>();
    let events = recorded.get::<BothLevels>(entity).unwrap();
    let event = events.last().unwrap();
    assert_eq!(
        event.state,
        ActionState::None,
        "if a required condition fails, it should override regular conditions"
    );
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        let down = Down::default();
        let released = Released::default();
        let blocked_by = BlockedBy::<Blocker>::default();
        let swizzle_axis = SwizzleAxis::YXZ;
        let negate = Negate::default();
        let scalar = Scalar::splat(2.0);

        ctx.bind::<Blocker>().with(Blocker::KEY);
        ctx.bind::<InputLevel>()
            .with(
                InputBind::new(InputLevel::KEY1)
                    .with_condition(blocked_by)
                    .with_condition(down)
                    .with_condition(released)
                    .with_modifier(swizzle_axis)
                    .with_modifier(scalar),
            )
            .with(
                InputBind::new(InputLevel::KEY2)
                    .with_condition(blocked_by)
                    .with_condition(down)
                    .with_condition(released)
                    .with_modifier(swizzle_axis)
                    .with_modifier(negate),
            );
        ctx.bind::<ActionLevel>()
            .with(ActionLevel::KEY1)
            .with(ActionLevel::KEY2)
            .with_condition(down)
            .with_condition(released)
            .with_condition(blocked_by)
            .with_modifier(swizzle_axis)
            .with_modifier(negate)
            .with_modifier(scalar);
        ctx.bind::<BothLevels>()
            .with(
                InputBind::new(BothLevels::KEY1)
                    .with_condition(down)
                    .with_modifier(scalar),
            )
            .with(
                InputBind::new(BothLevels::KEY2)
                    .with_condition(down)
                    .with_modifier(negate),
            )
            .with_condition(released)
            .with_condition(blocked_by)
            .with_modifier(swizzle_axis);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct InputLevel;

impl InputLevel {
    const KEY1: KeyCode = KeyCode::KeyA;
    const KEY2: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct ActionLevel;

impl ActionLevel {
    const KEY1: KeyCode = KeyCode::KeyC;
    const KEY2: KeyCode = KeyCode::KeyD;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct BothLevels;

impl BothLevels {
    const KEY1: KeyCode = KeyCode::KeyE;
    const KEY2: KeyCode = KeyCode::KeyF;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Blocker;

impl Blocker {
    const KEY: KeyCode = KeyCode::KeyG;
}
