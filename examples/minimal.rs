use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<OnFoot>()
        .add_input_context::<InCar>()
        .add_systems(Startup, spawn)
        .observe(walk)
        .observe(jump)
        .observe(enter_car)
        .observe(drive)
        .observe(exit_car)
        .run();
}

#[derive(Component)]
struct OnFoot;

impl InputContext for OnFoot {
    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Walk>().with_wasd();
        map.bind::<Jump>().with(KeyCode::Space);
        map.bind::<EnterCar>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug, InputAction)]
#[action_dim(Axis2D)]
struct Walk;

#[derive(Debug, InputAction)]
#[action_dim(Bool)]
struct Jump;

#[derive(Debug, InputAction)]
#[action_dim(Bool)]
struct EnterCar;

#[derive(Component)]
struct InCar;

impl InputContext for InCar {
    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Drive>().with_wasd();
        map.bind::<ExitCar>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug, InputAction)]
#[action_dim(Axis2D)]
struct Drive;

#[derive(Debug, InputAction)]
#[action_dim(Bool)]
struct ExitCar;

fn spawn(mut commands: Commands) {
    commands.spawn(OnFoot);
}

fn walk(trigger: Trigger<ActionEvent<Walk>>) {
    if let ActionEventKind::Fired {
        value, fired_secs, ..
    } = trigger.event().kind
    {
        info!("walking with direction `{value:?}` for `{fired_secs}` secs");
    }
}

fn jump(trigger: Trigger<ActionEvent<Jump>>) {
    if trigger.event().is_started() {
        info!("jumping in the air");
    }
}

fn enter_car(trigger: Trigger<ActionEvent<EnterCar>>, mut commands: Commands) {
    if trigger.event().is_started() {
        info!("entering car");
        commands
            .entity(trigger.entity())
            .remove::<OnFoot>()
            .insert(InCar);
    }
}

fn drive(trigger: Trigger<ActionEvent<Drive>>) {
    if let ActionEventKind::Fired {
        value, fired_secs, ..
    } = trigger.event().kind
    {
        info!("driving with direction `{value:?}` for `{fired_secs}` secs");
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
