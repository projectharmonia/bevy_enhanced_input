//! Simple setup with a single context.

mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBox, PlayerBoxBundle, PlayerBoxPlugin, DEFAULT_SPEED};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EnhancedInputPlugin,
            PlayerBoxPlugin,
            GamePlugin,
        ))
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<PlayerBox>() // All input contexts should be registered inside the app.
            .add_systems(Startup, Self::spawn)
            .observe(Self::apply_movement)
            .observe(Self::rotate);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());

        // Spawn an entity with a component that implements `InputContext`.
        commands.spawn(PlayerBoxBundle::default());
    }

    fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        // The value has already been preprocessed by defined modifiers.
        transform.translation += event.value.extend(0.0);
    }

    fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.rotate_z(FRAC_PI_4);
    }
}

// To define mappings for actions, implement the context trait.
// You can implement it for your character component directly, as
// shown in this example, if you don't plan to switch contexts.
impl InputContext for PlayerBox {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        // Create a context and start defining bindings.
        // Multiple inputs can be assigned to a single action,
        // and the action will respond to any of them.
        let mut ctx = ContextInstance::default();

        // Mappings like WASD or sticks are very common,
        // so we provide built-ins to assign all keys/axes at once.
        // We don't assign any conditions and in this case the action will
        // be triggered with any non-zero value.
        ctx.bind::<Move>()
            .to(WasdKeys)
            .to(GamepadStick::Left)
            .with_modifier(DeadZone::default()) // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            .with_modifier(DeltaLerp::default()) // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            .with_modifier(Scale::splat(DEFAULT_SPEED)); // Additionally multiply by a constant to achieve the desired speed.

        ctx.bind::<Rotate>()
            .to(KeyCode::Space)
            .to(GamepadButtonType::South);

        ctx
    }
}

// All actions should implement the `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
// The only necessary parameter is `dim`, which defines the output type.
#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;
