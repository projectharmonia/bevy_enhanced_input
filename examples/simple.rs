//! Simple setup with a single context.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin, GamePlugin))
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>() // All contexts should be registered inside the app.
            .add_systems(Startup, Self::spawn)
            .observe(Self::move_character)
            .observe(Self::jump);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        // To associate an entity with actions, insert the context.
        commands.spawn(Player);
    }

    fn move_character(trigger: Trigger<ActionEvent<Move>>) {
        if let ActionEventKind::Fired {
            value, fired_secs, ..
        } = trigger.event().kind
        {
            info!("moving with direction `{value:?}` for `{fired_secs}` secs");
        }
    }

    fn jump(trigger: Trigger<ActionEvent<Jump>>) {
        // If you are not interested in action value, we provide
        // methods to quickly check action kind on the event.
        if trigger.event().is_started() {
            info!("jumping in the air");
        }
    }
}

#[derive(Component)]
struct Player;

// To define mappings for actions, implement the context trait.
// Multiple inputs can be assigned to a single action,
// and the action will respond to any of them.
impl InputContext for Player {
    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        // Mappings like WASD or sticks are very common,
        // so we provide built-ins to assign all keys/axes at once.
        map.bind::<Move>()
            .with_wasd()
            .with_stick(GamepadStick::Left);

        // If you don't need keyboard modifiers, you can pass
        // buttons directly, thanks to the `From` impl.
        map.bind::<Jump>()
            .with(KeyCode::Space)
            .with(GamepadButtonType::South);

        map
    }
}

// All actions should implement `InputAction` trait.
// It can be done manually, but we provide a derive for convenience.
#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Jump;
