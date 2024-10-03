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
    fn context_map() -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Walk>().with_wasd();
        map.bind::<Jump>().with(KeyCode::Space);
        map.bind::<EnterCar>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug)]
struct Walk;

impl InputAction for Walk {
    const DIM: ActionValueDim = ActionValueDim::Axis2D;
}

#[derive(Debug)]
struct Jump;

impl InputAction for Jump {
    const DIM: ActionValueDim = ActionValueDim::Bool;
}

#[derive(Debug)]
struct EnterCar;

impl InputAction for EnterCar {
    const DIM: ActionValueDim = ActionValueDim::Bool;
}

#[derive(Component)]
struct InCar;

impl InputContext for InCar {
    fn context_map() -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<Drive>().with_wasd();
        map.bind::<ExitCar>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug)]
struct Drive;

impl InputAction for Drive {
    const DIM: ActionValueDim = ActionValueDim::Axis2D;
}

#[derive(Debug)]
struct ExitCar;

impl InputAction for ExitCar {
    const DIM: ActionValueDim = ActionValueDim::Bool;
}

fn spawn(mut commands: Commands) {
    commands.spawn(OnFoot);
}

fn walk(trigger: Trigger<ActionEvent<Walk>>) {
    if let ActionEventKind::Fired {
        value,
        fired_secs,
        elapsed_secs: _,
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
        value,
        fired_secs,
        elapsed_secs: _,
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
