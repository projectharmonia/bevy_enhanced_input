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
            .observe(Self::dive);
    }
}

impl GamePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
        commands.spawn(PlayerBoxBundle::default());
    }

    fn apply_movement(trigger: Trigger<ActionEvent<Move>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        if event.kind.is_fired() {
            let mut transform = players.get_mut(trigger.entity()).unwrap();
            transform.translation += event.value.as_axis3d();
        }
    }

    fn rotate(trigger: Trigger<ActionEvent<Rotate>>, mut players: Query<&mut Transform>) {
        let event = trigger.event();
        if event.kind.is_started() {
            let mut transform = players.get_mut(trigger.entity()).unwrap();
            transform.rotate_z(FRAC_PI_4);
        }
    }

    fn enter_water(
        trigger: Trigger<ActionEvent<EnterWater>>,
        mut commands: Commands,
        mut players: Query<&mut PlayerColor>,
    ) {
        let event = trigger.event();
        if event.kind.is_started() {
            // Change color for visibility.
            let mut color = players.get_mut(trigger.entity()).unwrap();
            color.0 = INDIGO_600.into();

            commands.entity(trigger.entity()).insert(Swimming);
        }
    }

    fn dive(trigger: Trigger<ActionEvent<Dive>>, mut players: Query<&mut Visibility>) {
        let event = trigger.event();

        // Hide player while diving.
        let target_visibility = match event.kind {
            ActionEventKind::Started => Visibility::Hidden,
            ActionEventKind::Completed { .. } => Visibility::Visible,
            _ => return,
        };

        let mut visibility = players.get_mut(trigger.entity()).unwrap();
        *visibility = target_visibility;
    }

    fn exit_water(
        trigger: Trigger<ActionEvent<ExitWater>>,
        mut commands: Commands,
        mut players: Query<&mut PlayerColor>,
    ) {
        let event = trigger.event();
        if event.kind.is_fired() {
            let mut color = players.get_mut(trigger.entity()).unwrap();
            color.0 = Default::default();

            commands.entity(trigger.entity()).remove::<Swimming>();
        }
    }
}

impl InputContext for PlayerBox {
    fn context_instance(_world: &World, _entity: Entity) -> ContextInstance {
        let mut ctx = ContextInstance::default();

        ctx.bind::<Move>()
            .with_wasd()
            .with_modifier(Normalize)
            .with_modifier(DeltaLerp::default())
            .with_modifier(Scale::splat(DEFAULT_SPEED));
        ctx.bind::<Rotate>().with(KeyCode::Space);
        ctx.bind::<EnterWater>().with(KeyCode::Enter);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Rotate;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
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
        ctx.bind::<Dive>().with(KeyCode::Space);
        ctx.bind::<ExitWater>().with(KeyCode::Enter);

        ctx
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Dive;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct ExitWater;
