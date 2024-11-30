use std::any;

use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn input_level() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(InputLevel::KEY1);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<InputLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::Y * 2.0).into());
    assert_eq!(action.state(), ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ChordMember::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<InputLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::Y * 2.0).into());
    assert_eq!(action.state(), ActionState::Fired);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(InputLevel::KEY1);
    keys.press(InputLevel::KEY2);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<InputLevel>().unwrap();
    assert_eq!(action.value(), Vec2::NEG_Y.into());
    assert_eq!(action.state(), ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(InputLevel::KEY1);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<InputLevel>().unwrap();
    assert_eq!(action.value(), Vec2::Y.into());
    assert_eq!(action.state(), ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<InputLevel>().unwrap();
    assert_eq!(action.value(), Vec2::ZERO.into());
    assert_eq!(
        action.state(),
        ActionState::None,
        "if a blocker condition fails, it should override other conditions"
    );

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(Blocker::KEY);
    keys.press(EventsBlocker::KEY);

    panic_on_action_events::<InputLevel>(app.world_mut());
    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<InputLevel>().unwrap();
    assert_eq!(action.value(), Vec2::Y.into());
    assert_eq!(action.state(), ActionState::Fired);
}

#[test]
fn action_level() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ActionLevel::KEY1);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<ActionLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::NEG_Y * 2.0).into());
    assert_eq!(action.state(), ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ChordMember::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<ActionLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::NEG_Y * 2.0).into());
    assert_eq!(action.state(), ActionState::Fired);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(ActionLevel::KEY1);
    keys.press(ActionLevel::KEY2);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<ActionLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::NEG_Y * 2.0).into());
    assert_eq!(action.state(), ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ActionLevel::KEY1);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<ActionLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::NEG_Y * 4.0).into());
    assert_eq!(action.state(), ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<ActionLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::NEG_Y * 4.0).into());
    assert_eq!(
        action.state(),
        ActionState::None,
        "if a blocker condition fails, it should override other conditions"
    );

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(Blocker::KEY);
    keys.press(EventsBlocker::KEY);
    panic_on_action_events::<ActionLevel>(app.world_mut());

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<ActionLevel>().unwrap();
    assert_eq!(action.value(), (Vec2::NEG_Y * 4.0).into());
    assert_eq!(action.state(), ActionState::Fired);
}

#[test]
fn both_levels() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(BothLevels::KEY1);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<BothLevels>().unwrap();
    assert_eq!(action.value(), (Vec2::Y * 2.0).into());
    assert_eq!(action.state(), ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ChordMember::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<BothLevels>().unwrap();
    assert_eq!(action.value(), (Vec2::Y * 2.0).into());
    assert_eq!(action.state(), ActionState::Fired);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(BothLevels::KEY1);
    keys.press(BothLevels::KEY2);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<BothLevels>().unwrap();
    assert_eq!(action.value(), Vec2::NEG_Y.into());
    assert_eq!(action.state(), ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(BothLevels::KEY1);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<BothLevels>().unwrap();
    assert_eq!(action.value(), Vec2::Y.into());
    assert_eq!(action.state(), ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<BothLevels>().unwrap();
    assert_eq!(action.value(), Vec2::Y.into());
    assert_eq!(
        action.state(),
        ActionState::None,
        "if a blocker condition fails, it should override other conditions"
    );

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(Blocker::KEY);
    keys.press(EventsBlocker::KEY);
    panic_on_action_events::<BothLevels>(app.world_mut());

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<BothLevels>().unwrap();
    assert_eq!(action.value(), Vec2::Y.into());
    assert_eq!(action.state(), ActionState::Fired);
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        let down = Press::default();
        let release = Release::default();
        let chord = Chord::<ChordMember>::default();
        let block_by = BlockBy::<Blocker>::default();
        let block_events_by = BlockBy::<EventsBlocker>::events_only();
        let swizzle_axis = SwizzleAxis::YXZ;
        let negate = Negate::default();
        let scale = Scale::splat(2.0);

        ctx.bind::<ChordMember>().to(ChordMember::KEY);
        ctx.bind::<Blocker>().to(Blocker::KEY);
        ctx.bind::<EventsBlocker>().to(EventsBlocker::KEY);
        ctx.bind::<InputLevel>().to((
            InputLevel::KEY1
                .with_conditions((chord, block_by, block_events_by, down, release))
                .with_modifiers((swizzle_axis, scale)),
            InputLevel::KEY2
                .with_conditions((chord, block_by, block_events_by, down, release))
                .with_modifiers((swizzle_axis, negate)),
        ));
        ctx.bind::<ActionLevel>()
            .to((ActionLevel::KEY1, ActionLevel::KEY2))
            .with_conditions((down, release, chord, block_by, block_events_by))
            .with_modifiers((swizzle_axis, negate, scale));
        ctx.bind::<BothLevels>()
            .to((
                BothLevels::KEY1.with_conditions(down).with_modifiers(scale),
                BothLevels::KEY2
                    .with_conditions(down)
                    .with_modifiers(negate),
            ))
            .with_conditions((release, chord, block_by, block_events_by))
            .with_modifiers(swizzle_axis);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct InputLevel;

impl InputLevel {
    const KEY1: KeyCode = KeyCode::KeyA;
    const KEY2: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct ActionLevel;

impl ActionLevel {
    const KEY1: KeyCode = KeyCode::KeyC;
    const KEY2: KeyCode = KeyCode::KeyD;
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct BothLevels;

impl BothLevels {
    const KEY1: KeyCode = KeyCode::KeyE;
    const KEY2: KeyCode = KeyCode::KeyF;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ChordMember;

impl ChordMember {
    const KEY: KeyCode = KeyCode::KeyG;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Blocker;

impl Blocker {
    const KEY: KeyCode = KeyCode::KeyH;
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct EventsBlocker;

impl EventsBlocker {
    const KEY: KeyCode = KeyCode::KeyI;
}

fn panic_on_action_events<A: InputAction>(world: &mut World) {
    world.add_observer(panic_on_event::<Started<A>>);
    world.add_observer(panic_on_event::<Ongoing<A>>);
    world.add_observer(panic_on_event::<Fired<A>>);
    world.add_observer(panic_on_event::<Completed<A>>);
    world.add_observer(panic_on_event::<Canceled<A>>);
}

fn panic_on_event<E: Event>(_trigger: Trigger<E>) {
    panic!(
        "event for action `{}` shouldn't trigger",
        any::type_name::<E>()
    );
}
