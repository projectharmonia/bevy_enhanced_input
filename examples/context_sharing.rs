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
        let event = trigger.event();
        let entity = trigger.entity();
        if let ActionEventKind::Fired { fired_secs, .. } = event.kind {
            info!(
                "entity `{entity}` moving with direction `{:?}` for `{fired_secs}` secs",
                event.value
            );
        }
    }

    fn jump(trigger: Trigger<ActionEvent<Jump>>) {
        let event = trigger.event();
        let entity = trigger.entity();
        if event.kind.is_started() {
            info!("entity `{entity}` jumping in the air");
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
    const MODE: ContextMode = ContextMode::Shared;

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>().with_wasd();
        ctx.bind::<Jump>().with(KeyCode::Space);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Jump;
