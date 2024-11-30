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

[`Conditions`](input_context::input_condition) define how an action activates. We also provide built-in conditions, such as [`Press`],
[`Release`], [`Hold`], etc. You can also add your own by implementing [`InputCondition`].

# Quick start

We provide a [`prelude`] module, which exports most of the typically used traits and types.

1. Add [`EnhancedInputPlugin`] to your app.
2. Define gameplay actions as unit structs and implement [`InputAction`] for them.
3. Define context components and assign actions to them by implementing [`InputContext`].
4. Register contexts using [`ContextAppExt::add_input_context`].
5. Insert contexts to entities you want to control.
6. Create observers to react on [`events`](crate::input_context::events) for each action.

For more details, see the documentation on relevant types. You can also find examples in the repository.

# Input and UI

Currently, Bevy does not have a focus management API. However, to prevent actions from being triggered
while interacting with the UI, we implement temporary workarounds enabled by specific cargo features:

* If the `ui_priority` feature is enabled, we check if any [`Interaction`] component is not [`Interaction::None`] and discard all mouse inputs.
* If the `egui_priority` feature is enabled, we check if any egui context requires keyboard or mouse input and discard those inputs accordingly.

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

// Required for the derive macro to work within the crate.
extern crate self as bevy_enhanced_input;

pub mod action_value;
pub mod input;
pub mod input_context;

pub mod prelude {
    pub use super::{
        action_value::{ActionValue, ActionValueDim},
        input::{GamepadDevice, Input, InputModKeys, ModKeys},
        input_context::{
            context_instance::{ActionBind, ContextInstance, GamepadStick, InputBind},
            events::*,
            input_action::{Accumulation, ActionData, ActionState, InputAction},
            input_condition::{
                block_by::*, chord::*, condition_timer::*, hold::*, hold_and_release::*,
                just_press::*, press::*, pulse::*, release::*, tap::*, ConditionKind,
                InputCondition,
            },
            input_modifier::{
                accumulate_by::*, dead_zone::*, delta_lerp::*, delta_scale::*,
                exponential_curve::*, negate::*, scale::*, swizzle_axis::*, InputModifier,
            },
            ContextAppExt, ContextInstances, ContextMode, InputContext, RebuildInputContexts,
        },
        EnhancedInputPlugin, EnhancedInputSystem,
    };
    pub use bevy_enhanced_input_macros::InputAction;
}

use bevy::{input::InputSystem, prelude::*};

use input::input_reader::InputReader;
use prelude::*;

/// Initializes contexts and feeds inputs to them.
pub struct EnhancedInputPlugin;

impl Plugin for EnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextInstances>()
            .configure_sets(PreUpdate, EnhancedInputSystem.after(InputSystem))
            .add_systems(PreUpdate, Self::update.in_set(EnhancedInputSystem));
    }
}

impl EnhancedInputPlugin {
    fn update(
        mut commands: Commands,
        mut reader: InputReader,
        time: Res<Time<Virtual>>, // We explicitly use `Virtual` to have access to `relative_speed`.
        mut instances: ResMut<ContextInstances>,
    ) {
        reader.update_state();
        instances.update(&mut commands, &mut reader, &time);
    }
}

/// Label for the system that updates input context instances.
///
/// Runs in [`PreUpdate`].
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct EnhancedInputSystem;
