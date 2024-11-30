//! One context applied on top of another and overrides some of the mappings.

mod player_box;

use std::f32::consts::FRAC_PI_4;

use bevy::{color::palettes::tailwind::INDIGO_600, prelude::*};
use bevy_enhanced_input::prelude::*;

use player_box::{PlayerBox, PlayerBoxBundle, PlayerBoxPlugin, PlayerColor, DEFAULT_SPEED};

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
        app.add_input_context::<PlayerBox>()
            .add_input_context::<Swimming>()
            .add_systems(Startup, Self::spawn)
            .observe(Self::apply_movement)
            .observe(Self::rotate)
            .observe(Self::exit_water)
            .observe(Self::enter_water)
            .observe(Self::start_diving)
            .observe(Self::end_diving);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
        commands.spawn(PlayerBoxBundle::default());
    }

    fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.translation += event.value.extend(0.0);
    }

    fn rotate(trigger: Trigger<Started<Rotate>>, mut players: Query<&mut Transform>) {
        let mut transform = players.get_mut(trigger.entity()).unwrap();
        transform.rotate_z(FRAC_PI_4);
    }

    fn enter_water(
        trigger: Trigger<Started<EnterWater>>,
        mut commands: Commands,
        mut players: Query<&mut PlayerColor>,
    ) {
        // Change color for visibility.
        let mut color = players.get_mut(trigger.entity()).unwrap();
        **color = INDIGO_600.into();

        commands.entity(trigger.entity()).insert(Swimming);
    }

    fn start_diving(trigger: Trigger<Started<Dive>>, mut players: Query<&mut Visibility>) {
        let mut visibility = players.get_mut(trigger.entity()).unwrap();
        *visibility = Visibility::Hidden;
    }

    fn end_diving(trigger: Trigger<Completed<Dive>>, mut players: Query<&mut Visibility>) {
        let mut visibility = players.get_mut(trigger.entity()).unwrap();
        *visibility = Visibility::Visible;
    }

    fn exit_water(
        trigger: Trigger<Fired<ExitWater>>,
        mut commands: Commands,
        mut players: Query<&mut PlayerColor>,
    ) {
        let mut color = players.get_mut(trigger.entity()).unwrap();
        **color = Default::default();

        commands.entity(trigger.entity()).remove::<Swimming>();
    }
}

impl InputContext for PlayerBox {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>()
            .to(Cardinal::wasd_keys())
            .with_modifiers((
                DeadZone::default(),
                DeltaLerp::default(),
                Scale::splat(DEFAULT_SPEED),
            ));
        ctx.bind::<Rotate>().to(KeyCode::Space);
        ctx.bind::<EnterWater>().to(KeyCode::Enter);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Rotate;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct EnterWater;

/// Context that overrides some actions from [`PlayerBox`].
#[derive(Component)]
struct Swimming;

impl InputContext for Swimming {
    const PRIORITY: isize = 1; // Set higher priority to execute its actions first.

    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        // `PlayerBox` has lower priority, so `Dive` and `ExitWater` consume inputs first,
        // preventing `Rotate` and `EnterWater` from being triggered.
        // The consuming behavior can be configured in the `InputAction` trait.
        ctx.bind::<Dive>().to(KeyCode::Space);
        ctx.bind::<ExitWater>().to(KeyCode::Enter);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Dive;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ExitWater;
