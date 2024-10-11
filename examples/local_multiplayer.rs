//! Two players that use the same context type, but with different mappings.

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
        app.add_input_context::<Player>()
            .add_systems(Startup, Self::spawn)
            .observe(Self::move_character)
            .observe(Self::jump);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn((Player, PlayerIndex(0)));
        commands.spawn((Player, PlayerIndex(1)));
    }

    fn move_character(trigger: Trigger<ActionEvent<Move>>, players: Query<&PlayerIndex>) {
        let event = trigger.event();
        if let ActionEventKind::Fired { fired_secs, .. } = event.kind {
            let index = **players.get(trigger.entity()).unwrap();
            info!(
                "player {index} moving with direction `{:?}` for `{fired_secs}` secs",
                event.value
            );
        }
    }

    fn jump(trigger: Trigger<ActionEvent<Jump>>, players: Query<&PlayerIndex>) {
        let event = trigger.event();
        if event.kind.is_started() {
            let index = **players.get(trigger.entity()).unwrap();
            info!("player {index} jumping in the air");
        }
    }
}

#[derive(Component, Deref)]
struct PlayerIndex(usize);

#[derive(Component)]
struct Player;

impl InputContext for Player {
    fn context_map(world: &World, entity: Entity) -> ContextMap {
        // Could be stored in the context itself, but it's usually
        // better to have a separate component that is shared
        // across all contexts.
        let index = **world.get::<PlayerIndex>(entity).unwrap();

        // By default context read inputs from all gamepads,
        // but for local multiplayer we need assign specific
        // gamepad index.
        let mut map = ContextMap::with_gamepad(index);

        // Assign different mappings based player index.
        match index {
            0 => {
                map.bind::<Move>()
                    .with_wasd()
                    .with_stick(GamepadStick::Left);
                map.bind::<Jump>()
                    .with(KeyCode::Space)
                    .with(GamepadButtonType::South);
            }
            1 => {
                map.bind::<Move>()
                    .with_arrows()
                    .with_stick(GamepadStick::Left);
                map.bind::<Jump>()
                    .with(KeyCode::Numpad0)
                    .with(GamepadButtonType::South);
            }
            _ => {
                panic!("game expects only 2 players");
            }
        }

        map
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Jump;
