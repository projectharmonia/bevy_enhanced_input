//! Two entities that share a single context.
//! This could be used for games where you control multiple characters at the same time,
//! such as Binary Land for NES.

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
        // Spawn two entities with the same context.
        commands.spawn(Player);
        commands.spawn(Player);
    }

    fn move_character(trigger: Trigger<ActionEvent<Move>>) {
        if let ActionEventKind::Fired {
            value, fired_secs, ..
        } = trigger.event().kind
        {
            info!(
                "entity `{}` moving with direction `{value:?}` for `{fired_secs}` secs",
                trigger.entity()
            );
        }
    }

    fn jump(trigger: Trigger<ActionEvent<Jump>>) {
        if trigger.event().is_started() {
            info!("entity `{}` jumping in the air", trigger.entity());
        }
    }
}

#[derive(Component)]
struct Player;

impl InputContext for Player {
    // By default all context instances are processed individually.
    // This means if multiple entities spawned with the same mappings,
    // actions from the first processed context may consume inputs.
    // Make it shared to have a single context instance for all entities
    // with this context.
    const KIND: ContextKind = ContextKind::Shared;

    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Move>().with_wasd();
        map.bind::<Jump>().with(KeyCode::Space);

        map
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Jump;
