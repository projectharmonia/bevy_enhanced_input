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
        let event = trigger.event();
        if let ActionTransition::Fired { fired_secs, .. } = event.transition {
            info!(
                "moving with direction `{:?}` for `{fired_secs}` secs",
                event.value
            );
        }
    }

    fn jump(trigger: Trigger<ActionEvent<Jump>>) {
        let event = trigger.event();
        if event.transition.is_started() {
            info!("jumping in the air");
        }
    }

    fn enter_water(trigger: Trigger<ActionEvent<EnterWater>>, mut commands: Commands) {
        let event = trigger.event();
        if event.transition.is_started() {
            info!("entering water");
            commands.entity(trigger.entity()).insert(Swimming);
        }
    }

    fn dive(trigger: Trigger<ActionEvent<Dive>>) {
        let event = trigger.event();
        if event.transition.is_started() {
            info!("diving");
        }
    }

    fn exit_water(trigger: Trigger<ActionEvent<ExitWater>>, mut commands: Commands) {
        let event = trigger.event();
        if event.transition.is_fired() {
            info!("exiting water");
            commands.entity(trigger.entity()).remove::<Swimming>();
        }
    }
}

#[derive(Component)]
struct Player;

impl InputContext for Player {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>().with_wasd();
        ctx.bind::<Jump>().with(KeyCode::Space);
        ctx.bind::<EnterWater>().with(KeyCode::Enter);

        ctx
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

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut map = ContextInstance::default();

        // `Player` has lower priority, so `Dive` and `ExitWater` consume inputs first,
        // preventing `Jump` and `EnterWater` from being triggered.
        // The consuming behavior can be configured in the `InputAction` trait.
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
