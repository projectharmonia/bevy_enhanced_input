/*!
Dynamic and contextual input mappings inspired by [Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine) for Bevy.

# What makes Enhanced Input... enhanced?

Instead of directly reacting to inputs from various sources (like keyboard, gamepad, etc.), you assign inputs to gameplay actions
like `Move` or `Jump`, which are just unit structs markers. Actions are assigned to contexts, which are components that represent the current
state of the player character, like `OnFoot` or `InCar`.

For example, if you have a player character that can be on foot or drive a car, you can swap the context to have the same keys
perform different actions. On foot, pressing Space makes the character jump, while when driving, the same key acts as a brake.

# Core concepts

Entities can have any number of contexts, with evaluation order controlled by a defined priority. Actions can also consume inputs,
allowing you to layer behaviors on top of each other.

Instead of reacting to raw input data like "Released" or "Pressed", the crate provides modifiers and conditions.

[`Modifiers`](input_context::input_modifier) let you change the input before passing it to the action. We provide common modifiers,
like [`DeadZone`], [`Negate`], etc., but you can add your own by implementing [`InputModifier`].

[`Conditions`](input_context::input_condition) define how an action activates. We also provide built-in conditions, such as [`Pressed`],
[`Released`], [`Hold`], etc. You can also add your game-specific conditions like `CanJump` by implementing [`InputCondition`].

# Quick start

We provide a [`prelude`] module, which exports most of the typically used traits and types.

1. Add [`EnhancedInputPlugin`] to your app.
2. Define gameplay actions as unit structs and implement [`InputAction`] for them.
3. Define context components and assign actions to them by implementing [`InputContext`].
4. Register contexts using [`ContextAppExt::add_input_context`].
5. Insert contexts to entities you want to control.
6. Create observers to react on [`ActionEvent`] for each action.

For more details, see the documentation on relevant types. You can also find examples in the repository.

# Troubleshooting

If you face any issue, try to enable logging to see what is going on.
To enable logging, you can temporarely set `RUST_LOG` environment variable to `bevy_enhanced_input=debug`
(or `bevy_enhanced_input=trace` for more noisy output) like this:

```bash
RUST_LOG=bevy_enhanced_input=debug cargo run
```

The exact method depends on the OS shell.

Alternatively you can configure [`LogPlugin`](bevy::log::LogPlugin) to make it permanent.
*/

pub mod action_value;
pub mod input;
pub mod input_context;

pub mod prelude {
    pub use super::{
        action_value::{ActionValue, ActionValueDim},
        input::{GamepadDevice, Input, Modifiers},
        input_context::{
            context_instance::{ActionBind, ContextInstance, GamepadStick, InputMap},
            input_action::{Accumulation, ActionEvent, ActionTransition, InputAction},
            input_condition::{
                blocked_by::*, chord::*, down::*, held_timer::*, hold::*, hold_and_release::*,
                pressed::*, pulse::*, released::*, tap::*, ConditionKind, InputCondition,
            },
            input_modifier::{
                dead_zone::*, exponential_curve::*, negate::*, normalize::*, scalar::*,
                scale_by_delta::*, smooth_delta::*, swizzle_axis::*, InputModifier,
            },
            ContextAppExt, ContextMode, InputContext,
        },
        EnhancedInputPlugin,
    };
    pub use bevy_enhanced_input_macros::InputAction;
}

use bevy::{ecs::system::SystemState, input::InputSystem, prelude::*};

use input::input_reader::InputReader;
use input_context::ContextInstances;
#[allow(unused_imports, reason = "used in the docs")]
use prelude::*;

/// Initializes contexts and feeds inputs to them.
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
