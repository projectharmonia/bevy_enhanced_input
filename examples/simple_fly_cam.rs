//! Simple fly camera with a single context.

use bevy::{prelude::*, window::CursorGrabMode};
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<FlyCam>() // All contexts should be registered.
        .add_observer(binding) // Add observer to setup bindings.
        .add_observer(apply_movement)
        .add_observer(capture_cursor)
        .add_observer(release_cursor)
        .add_observer(rotate)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Single<&mut Window>,
) {
    // Capture mouse.
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;

    // Spawn a camera with `Actions` component.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Actions::<FlyCam>::default(),
    ));

    // Setup simple 3D scene.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(25.0)))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

// To define bindings for actions, write an observer for `Binding`.
// It's also possible to create bindings before the insertion,
// but this way you can conveniently reload bindings when settings change.
fn binding(trigger: Trigger<Binding<FlyCam>>, mut players: Query<&mut Actions<FlyCam>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();

    // Bindings like WASD or sticks are very common,
    // so we provide built-ins to assign all keys/axes at once.
    // We don't assign any conditions and in this case the action will
    // be triggered with any non-zero value.
    // An action can have multiple inputs bound to it
    // and will respond to any of them.
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers((
            DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            Scale::splat(0.3), // Additionally multiply by a constant to achieve the desired speed.
        ));

    actions.bind::<Rotate>().to((
        // You can attach modifiers to individual inputs as well.
        Input::mouse_motion().with_modifiers((Scale::splat(0.1), Negate::all())),
        Axial::right_stick().with_modifiers_each((Scale::splat(2.0), Negate::x())),
    ));

    actions.bind::<CaptureCursor>().to(MouseButton::Left);
    actions.bind::<ReleaseCursor>().to(KeyCode::Escape);
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut transforms: Query<&mut Transform>) {
    let mut transform = transforms.get_mut(trigger.target()).unwrap();

    // Move to the camera direction.
    let rotation = transform.rotation;

    // Movement consists of X and -Z components, so swap Y and Z with negation.
    // We could do it with modifiers, but it wold be weird for an action to return
    // a `Vec3` like this, so we doing it inside the function.
    let mut movement = trigger.value.extend(0.0).xzy();
    movement.z = -movement.z;

    transform.translation += rotation * movement
}

fn rotate(
    trigger: Trigger<Fired<Rotate>>,
    mut players: Query<&mut Transform>,
    window: Single<&Window>,
) {
    if window.cursor_options.visible {
        return;
    }

    let mut transform = players.get_mut(trigger.target()).unwrap();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

    yaw += trigger.value.x.to_radians();
    pitch += trigger.value.y.to_radians();

    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn capture_cursor(_trigger: Trigger<Completed<CaptureCursor>>, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;
}

fn release_cursor(_trigger: Trigger<Completed<ReleaseCursor>>, mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

// Since it's possible to have multiple `Actions` components, you need
// to define a marker and derive `InputContext` trait.
#[derive(InputContext)]
struct FlyCam;

// All actions should implement the `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
// The only necessary parameter is `output`, which defines the output type.
#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct CaptureCursor;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ReleaseCursor;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Rotate;
