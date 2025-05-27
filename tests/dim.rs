use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn bool() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Bool::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Bool>()?, true.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Bool::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Bool>()?, false.into());

    Ok(())
}

#[test]
fn axis1d() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis1D::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Axis1D>()?, 1.0.into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis1D::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Axis1D>()?, 0.0.into());

    Ok(())
}

#[test]
fn axis2d() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis2D::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Axis2D>()?, (1.0, 0.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis2D::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Axis2D>()?, Vec2::ZERO.into());

    Ok(())
}

#[test]
fn axis3d() -> Result<()> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<Test>()
        .add_observer(binding)
        .finish();

    let entity = app.world_mut().spawn(Actions::<Test>::default()).id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis3D::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Axis3D>()?, (1.0, 0.0, 0.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis3D::KEY);

    app.update();

    let actions = app.world().get::<Actions<Test>>(entity).unwrap();
    assert_eq!(actions.value::<Axis3D>()?, Vec3::ZERO.into());

    Ok(())
}

fn binding(trigger: Trigger<Binding<Test>>, mut actions: Query<&mut Actions<Test>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Bool>().to(Bool::KEY);
    actions.bind::<Axis1D>().to(Axis1D::KEY);
    actions.bind::<Axis2D>().to(Axis2D::KEY);
    actions.bind::<Axis3D>().to(Axis3D::KEY);
}

#[derive(InputContext)]
struct Test;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Bool;

impl Bool {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(Debug, InputAction)]
#[input_action(output = f32)]
struct Axis1D;

impl Axis1D {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Axis2D;

impl Axis2D {
    const KEY: KeyCode = KeyCode::KeyC;
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec3)]
struct Axis3D;

impl Axis3D {
    const KEY: KeyCode = KeyCode::KeyD;
}
