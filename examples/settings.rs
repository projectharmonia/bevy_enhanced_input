//! Reading settings and rebinding keys.

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin, GamePlugin))
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputSettings>()
            .add_input_context::<Player>()
            .add_systems(Startup, Self::spawn)
            .add_systems(Update, Self::binding)
            .observe(Self::move_character)
            .observe(Self::jump)
            .observe(Self::start_binding)
            .observe(Self::finish_binding);
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

    fn start_binding(
        trigger: Trigger<ActionEvent<StartBinding>>,
        mut commands: Commands,
        players: Query<Entity, With<Player>>,
    ) {
        if trigger.event().is_completed() {
            info!("starting binding");
            commands.spawn(BindMenu);
            commands.entity(players.single()).despawn();
        }
    }

    fn binding(
        mut binding: Local<Option<ActiveBinding>>,
        mut commands: Commands,
        mut keyboard_events: EventReader<KeyboardInput>,
        mut settings: ResMut<InputSettings>,
        menus: Query<(Entity, Ref<BindMenu>)>,
    ) {
        let Ok((menu_entity, menu)) = menus.get_single() else {
            return;
        };

        if menu.is_added() {
            // Disable a message on open and reset the state.
            info!("press WASD to rebind directions or Space to rebind jump");
            *binding = None;
        }

        let Some(pressed_key) = keyboard_events
            .read()
            .filter(|input| input.state == ButtonState::Pressed)
            .map(|input| input.key_code)
            .next()
        else {
            return;
        };

        if let Some(binding) = *binding {
            // Bind the previosly selected key.
            match binding {
                ActiveBinding::UpKey => settings.up = pressed_key,
                ActiveBinding::LeftKey => settings.left = pressed_key,
                ActiveBinding::DownKey => settings.down = pressed_key,
                ActiveBinding::RightKey => settings.right = pressed_key,
                ActiveBinding::JumpKey => settings.jump = pressed_key,
            }

            commands.entity(menu_entity).despawn();

            // A new entity will be spawned with updated context.
            // You can also re-insert the component or
            // trigger `RebuildInputContexts` event to update all active contexts.
            commands.spawn(Player);
        } else {
            // Select a key to bind.
            let selected_binding = match pressed_key {
                KeyCode::KeyW => ActiveBinding::UpKey,
                KeyCode::KeyA => ActiveBinding::LeftKey,
                KeyCode::KeyS => ActiveBinding::DownKey,
                KeyCode::KeyD => ActiveBinding::RightKey,
                KeyCode::Space => ActiveBinding::JumpKey,
                _ => {
                    info!("unexpected key `{pressed_key:?}`, you need to press WASD or Space");
                    return;
                }
            };

            *binding = Some(selected_binding);
        }
    }

    fn finish_binding(
        trigger: Trigger<ActionEvent<FinishBinding>>,
        mut commands: Commands,
        menus: Query<Entity, With<BindMenu>>,
    ) {
        if trigger.event().is_completed() {
            info!("exiting settings");
            commands.entity(menus.single()).despawn();
            commands.spawn(Player);
        }
    }
}

#[derive(Component)]
struct Player;

impl InputContext for Player {
    fn context_map(world: &World, _entity: Entity) -> ContextMap {
        let settings = world.resource::<InputSettings>();

        let mut map = ContextMap::default();

        // Read keys and modifier parameters from settings.
        map.bind::<Move>()
            .with_axis2d([settings.up, settings.left, settings.down, settings.right])
            .with_modifier(Negate::all(settings.inverse));
        map.bind::<Jump>().with(settings.jump);

        // Some actions could be non-configrable.
        map.bind::<StartBinding>().with(KeyCode::Enter);

        map
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Axis2D)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct Jump;

/// Activates binding context.
#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct StartBinding;

/// Context with a single action to stop binding.
///
/// We ommited actual UI for the sake of simplicity.
#[derive(Component)]
struct BindMenu;

impl InputContext for BindMenu {
    fn context_map(_world: &World, _entity: Entity) -> ContextMap {
        let mut map = ContextMap::default();

        map.bind::<FinishBinding>().with(KeyCode::Escape);

        map
    }
}

#[derive(Debug, InputAction)]
#[input_action(dim = Bool)]
struct FinishBinding;

/// Currently binding settings parameter.
///
/// Used to replace settings UI.
#[derive(Debug, Clone, Copy)]
enum ActiveBinding {
    UpKey,
    LeftKey,
    DownKey,
    RightKey,
    JumpKey,
}

#[derive(Resource)]
struct InputSettings {
    up: KeyCode,
    left: KeyCode,
    down: KeyCode,
    right: KeyCode,
    jump: KeyCode,
    inverse: bool,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            up: KeyCode::KeyW,
            left: KeyCode::KeyA,
            down: KeyCode::KeyS,
            right: KeyCode::KeyD,
            jump: KeyCode::Space,
            inverse: false,
        }
    }
}
