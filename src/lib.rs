pub mod action_value;
pub mod input_context;
pub mod input_reader;

pub mod prelude {
    pub use super::{
        action_value::{ActionValue, ActionValueDim},
        input_context::{
            context_map::{ActionMap, ContextMap, GamepadStick, InputMap},
            input_action::{Accumulation, ActionEvent, ActionEventKind, InputAction},
            input_condition::*,
            input_modifier::*,
            ContextAppExt, ContextKind, InputContext,
        },
        input_reader::{GamepadDevice, Input, InputReader, KeyboardModifiers},
        EnhancedInputPlugin,
    };
    pub use bevy_enhanced_input_macros::InputAction;
}

use bevy::{ecs::system::SystemState, input::InputSystem, prelude::*};

use input_context::InputContexts;
use input_reader::UiInput;
use prelude::*;

pub struct EnhancedInputPlugin;

impl Plugin for EnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputContexts>()
            .add_systems(PreUpdate, Self::update.after(InputSystem));
    }
}

impl EnhancedInputPlugin {
    fn update(
        world: &mut World,
        state: &mut SystemState<(Commands, InputReader, UiInput, Res<Time>)>,
    ) {
        world.resource_scope(|world, mut contexts: Mut<InputContexts>| {
            let (mut commands, mut reader, ui_input, time) = state.get(world);
            reader.set_ignore_keyboard(ui_input.wants_keyboard());
            reader.set_ignore_mouse(ui_input.wants_mouse());
            reader.update_state();

            let delta = time.delta_seconds();

            contexts.update(world, &mut commands, &mut reader, delta);
        });

        state.apply(world);
    }
}
