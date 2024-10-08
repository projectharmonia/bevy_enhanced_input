use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin, SimplePlugin))
        .run();
}

struct SimplePlugin;

impl Plugin for SimplePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<OnFoot>()
            .add_input_context::<InCar>()
            .add_systems(Startup, Self::spawn)
            .observe(Self::move_character)
            .observe(Self::jump)
            .observe(Self::brake)
            .observe(Self::enter_car)
            .observe(Self::exit_car);
    }
}

impl SimplePlugin {
    fn spawn(mut commands: Commands) {
        commands.spawn(OnFoot);
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

    fn brake(trigger: Trigger<ActionEvent<Brake>>) {
        if trigger.event().is_fired() {
            info!("holding brake");
        }
    }

    fn enter_car(trigger: Trigger<ActionEvent<Crouch>>, mut commands: Commands) {
        if trigger.event().is_started() {
            info!("entering car");
            commands
                .entity(trigger.entity())
                .remove::<OnFoot>()
                .insert(InCar);
        }
    }

    fn exit_car(trigger: Trigger<ActionEvent<ExitCar>>, mut commands: Commands) {
        if trigger.event().is_started() {
            info!("exiting car");
            commands
                .entity(trigger.entity())
                .remove::<InCar>()
                .insert(OnFoot);
        }
    }
}

#[derive(Component)]
struct OnFoot;

impl InputContext for OnFoot {
    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Move>().with_wasd();
        map.bind::<Jump>().with(KeyCode::Space);
        map.bind::<Crouch>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug, InputAction)]
#[action_dim(Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[action_dim(Bool)]
struct Jump;

#[derive(Debug, InputAction)]
#[action_dim(Bool)]
struct Crouch;

#[derive(Component)]
struct InCar;

impl InputContext for InCar {
    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Move>().with_wasd();
        map.bind::<ExitCar>().with(KeyCode::Enter);
        map.bind::<Brake>().with(KeyCode::Space);

        map
    }
}

#[derive(Debug, InputAction)]
#[action_dim(Bool)]
struct ExitCar;

#[derive(Debug, InputAction)]
#[action_dim(Bool)]
struct Brake;
