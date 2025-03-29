/*!
Dynamic and contextual input mappings inspired by [Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine) for Bevy.

# What makes Enhanced Input... enhanced?

Instead of directly reacting to inputs from various sources (like keyboard, gamepad, etc.), you assign inputs to gameplay actions
like `Move` or `Jump`, which are just unit structs markers. Actions are assigned to contexts, which are components that represent the current
state of the player character, like `OnFoot` or `InCar`.

For example, if you have a player character that can be on foot or drive a car, you can swap the context to have the same keys
perform different actions. On foot, pressing Space makes the character jump, while when driving, the same key acts as a brake.

Entities can have any number of contexts, with evaluation order controlled by a defined priority. Actions can also consume inputs,
allowing you to layer behaviors on top of each other.

Instead of reacting to raw input data like "Released" or "Pressed", the crate provides modifiers and conditions.

[`Modifiers`](input_modifier) let you change the input before passing it to the action. We provide common modifiers,
like [`DeadZone`], [`Negate`], etc., but you can add your own by implementing [`InputModifier`].

[`Conditions`](input_condition) define how an action activates. We also provide built-in conditions, such as [`Press`],
[`Release`], [`Hold`], etc. You can also add your own by implementing [`InputCondition`].

# Quick start

We provide a [`prelude`] module, which exports most of the typically used traits and types.

1. Add [`EnhancedInputPlugin`] to your app.
2. Define gameplay actions as unit structs and implement [`InputAction`] for them.
3. Define context components and assign actions to them by writing observers for [`Binding`]
4. Register markers using [`ActionsMarkerAppExt::add_actions_marker`].
5. Insert actions to entities you want to control.
6. Create observers to react on [`events`] for each action.

For more details, see the documentation on relevant types. You can also find examples in the repository.

# Input and UI

Currently, Bevy does not have a focus management API. However, to prevent actions from being triggered
while interacting with the UI, we implement temporary workarounds enabled by specific cargo features:

* If the `ui_priority` feature is enabled, we check if any [`Interaction`] component is not [`Interaction::None`] and discard all mouse inputs.
* If the `egui_priority` feature is enabled, we check if any egui context requires keyboard or mouse input and discard those inputs accordingly.

# Troubleshooting

If you face any issue, try to enable logging to see what is going on.
To enable logging, you can temporarily set `RUST_LOG` environment variable to `bevy_enhanced_input=debug`
(or `bevy_enhanced_input=trace` for more noisy output) like this:

```bash
RUST_LOG=bevy_enhanced_input=debug cargo run
```

The exact method depends on the OS shell.

Alternatively you can configure [`LogPlugin`](bevy::log::LogPlugin) to make it permanent.
*/

#![no_std]

extern crate std;

extern crate alloc;

// Required for the derive macro to work within the crate.
extern crate self as bevy_enhanced_input;

pub mod action_instances;
pub mod action_value;
pub mod actions;
pub mod events;
pub mod input;
pub mod input_action;
pub mod input_bind;
pub mod input_condition;
pub mod input_modifier;
mod input_reader;
pub mod preset;
mod trigger_tracker;

pub mod prelude {
    pub use super::{
        EnhancedInputPlugin, EnhancedInputSystem,
        action_instances::{ActionsMarkerAppExt, Binding, RebuildBindings},
        action_value::{ActionValue, ActionValueDim},
        actions::{ActionBind, ActionData, ActionState, Actions, ActionsMarker},
        events::*,
        input::{GamepadDevice, Input, InputModKeys, ModKeys},
        input_action::{Accumulation, InputAction},
        input_bind::{InputBind, InputBindModCond, InputBindSet},
        input_condition::{
            ConditionKind, InputCondition, block_by::*, chord::*, condition_timer::*, hold::*,
            hold_and_release::*, just_press::*, press::*, pulse::*, release::*, tap::*,
        },
        input_modifier::{
            InputModifier, accumulate_by::*, dead_zone::*, delta_scale::*, exponential_curve::*,
            negate::*, scale::*, smooth_nudge::*, swizzle_axis::*,
        },
        preset::{Bidirectional, Cardinal, GamepadStick},
    };
    pub use bevy_enhanced_input_macros::{ActionsMarker, InputAction};
}

use bevy::{
    ecs::{
        system::{ParamBuilder, QueryParamBuilder},
        world::FilteredEntityMut,
    },
    input::InputSystem,
    prelude::*,
};

use action_instances::{ActionInstances, ActionsRegistry};
use input_reader::{InputReader, ResetInput};
use prelude::*;

/// Initializes contexts and feeds inputs to them.
///
/// See also [`EnhancedInputSystem`].
pub struct EnhancedInputPlugin;

impl Plugin for EnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionInstances>()
            .init_resource::<ActionsRegistry>()
            .init_resource::<ResetInput>()
            .configure_sets(PreUpdate, EnhancedInputSystem.after(InputSystem));
    }

    fn finish(&self, app: &mut App) {
        let registry = app
            .world_mut()
            .remove_resource::<ActionsRegistry>()
            .expect("registry should be inserted in `build`");

        let update = (
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder.optional(|builder| {
                    for &id in registry.iter() {
                        builder.mut_id(id);
                    }
                });
            }),
        )
            .build_state(&mut app.world_mut())
            .build_system(update);

        let rebuild = (
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder.optional(|builder| {
                    for &id in registry.iter() {
                        builder.mut_id(id);
                    }
                });
            }),
        )
            .build_state(&mut app.world_mut())
            .build_any_system(rebuild);

        app.add_observer(rebuild)
            .add_systems(PreUpdate, update.in_set(EnhancedInputSystem));
    }
}

fn update(
    mut commands: Commands,
    mut reader: InputReader,
    time: Res<Time<Virtual>>, // We explicitly use `Virtual` to have access to `relative_speed`.
    mut instances: ResMut<ActionInstances>,
    mut actions: Query<FilteredEntityMut>,
) {
    reader.update_state();
    instances.update(&mut commands, &mut reader, &time, &mut actions);
}

fn rebuild(
    _trigger: Trigger<RebuildBindings>,
    mut commands: Commands,
    mut reset_input: ResMut<ResetInput>,
    mut instances: ResMut<ActionInstances>,
    time: Res<Time<Virtual>>,
    mut actions: Query<FilteredEntityMut>,
) {
    instances.rebuild(&mut commands, &mut reset_input, &time, &mut actions);
}

/// Label for the system that updates input context instances.
///
/// Runs in [`PreUpdate`].
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct EnhancedInputSystem;
