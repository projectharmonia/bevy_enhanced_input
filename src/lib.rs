pub mod action_value;
pub mod input_context;
pub mod input_reader;

pub mod prelude {
    pub use super::{
        action_value::{ActionValue, ActionValueDim},
        input_context::{
            context_instance::{ActionMap, ContextInstance, GamepadStick, InputMap},
            input_action::{Accumulation, ActionEvent, ActionEventKind, InputAction},
            input_condition::*,
            input_modifier::*,
            ContextAppExt, ContextMode, InputContext,
        },
        input_reader::{GamepadDevice, Input, KeyboardModifiers},
        EnhancedInputPlugin,
    };
    pub use bevy_enhanced_input_macros::InputAction;
}

use bevy::{ecs::system::SystemState, input::InputSystem, prelude::*};

use input_context::ContextInstances;
use input_reader::InputReader;

pub struct EnhancedInputPlugin;

impl Plugin for EnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextInstances>()
            .add_systems(PreUpdate, Self::update.after(InputSystem));
    }
}

impl EnhancedInputPlugin {
    fn update(world: &mut World, state: &mut SystemState<(Commands, InputReader, Res<Time>)>) {
        world.resource_scope(|world, mut contexts: Mut<ContextInstances>| {
            let (mut commands, mut reader, time) = state.get(world);
            reader.update_state();

            let delta = time.delta_seconds();

            contexts.update(world, &mut commands, &mut reader, delta);
        });

        state.apply(world);
    }
}
