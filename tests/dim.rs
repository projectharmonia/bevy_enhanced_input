use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

#[test]
fn bool() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Bool::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Bool>().unwrap();
    assert_eq!(action.value(), true.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Bool::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Bool>().unwrap();
    assert_eq!(action.value(), false.into());
}

#[test]
fn axis1d() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis1D::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Axis1D>().unwrap();
    assert_eq!(action.value(), 1.0.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis1D::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Axis1D>().unwrap();
    assert_eq!(action.value(), 0.0.into());
}

#[test]
fn axis2d() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis2D::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Axis2D>().unwrap();
    assert_eq!(action.value(), (1.0, 0.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis2D::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Axis2D>().unwrap();
    assert_eq!(action.value(), Vec2::ZERO.into());
}

#[test]
fn axis3d() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<DummyContext>();

    let entity = app.world_mut().spawn(DummyContext).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis3D::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Axis3D>().unwrap();
    assert_eq!(action.value(), (1.0, 0.0, 0.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis3D::KEY);

    app.update();

    let instances = app.world().resource::<ContextInstances>();
    let ctx = instances.get::<DummyContext>(entity).unwrap();
    let action = ctx.action::<Axis3D>().unwrap();
    assert_eq!(action.value(), Vec3::ZERO.into());
}

#[derive(Debug, Component)]
struct DummyContext;

impl InputContext for DummyContext {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Bool>().to(Bool::KEY);
        ctx.bind::<Axis1D>().to(Axis1D::KEY);
        ctx.bind::<Axis2D>().to(Axis2D::KEY);
        ctx.bind::<Axis3D>().to(Axis3D::KEY);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Bool;

impl Bool {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis1D)]
struct Axis1D;

impl Axis1D {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Axis2D;

impl Axis2D {
    const KEY: KeyCode = KeyCode::KeyC;
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis3D)]
struct Axis3D;

impl Axis3D {
    const KEY: KeyCode = KeyCode::KeyD;
}
