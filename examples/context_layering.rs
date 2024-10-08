//! One context applied on top of another and overrides some of the mappings.

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
            .add_input_context::<Swimming>()
            .add_systems(Startup, Self::spawn)
            .observe(Self::move_character)
            .observe(Self::jump)
            .observe(Self::exit_water)
            .observe(Self::enter_water)
            .observe(Self::dive);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
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
        if trigger.event().is_started() {
            info!("jumping in the air");
        }
    }

    fn enter_water(trigger: Trigger<ActionEvent<EnterWater>>, mut commands: Commands) {
        if trigger.event().is_started() {
            info!("entering water");
            commands.entity(trigger.entity()).insert(Swimming);
        }
    }

    fn dive(trigger: Trigger<ActionEvent<Dive>>) {
        if trigger.event().is_started() {
            info!("diving");
        }
    }

    fn exit_water(trigger: Trigger<ActionEvent<ExitWater>>, mut commands: Commands) {
        if trigger.event().is_fired() {
            info!("exiting water");
            commands.entity(trigger.entity()).remove::<Swimming>();
        }
    }
}

#[derive(Component)]
struct Player;

impl InputContext for Player {
    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Move>().with_wasd();
        map.bind::<Jump>().with(KeyCode::Space);
        map.bind::<EnterWater>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Jump;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct EnterWater;

/// Context that overrides some actions from [`Player`].
#[derive(Component)]
struct Swimming;

impl InputContext for Swimming {
    const PRIORITY: usize = 1; // Set higher priority to execute its actions first.

    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        // Dive and exit actions will consume, the input preventing
        // lower level priorities to see the input.
        // Consuming behavior is configurable in the `InputAction` trait.
        map.bind::<Dive>().with(KeyCode::Space);
        map.bind::<ExitWater>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Dive;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct ExitWater;
