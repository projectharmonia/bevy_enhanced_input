/*!
Input manager for [Bevy](https://bevyengine.org), inspired by
[Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine).

# Quick start

## Prelude

We provide a [`prelude`] module, which exports most of the typically used traits and types.

## Plugins

Add [`EnhancedInputPlugin`] to your app:

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

let mut app = App::new();
app.add_plugins((MinimalPlugins, EnhancedInputPlugin));
```

## Defining actions

Actions represent something that the user can do, like "Crouch" or "Fire Weapon". They are
represented by unit structs that implement the [`InputAction`] trait. Each action has an
associated [`InputAction::Output`] type. It’s the value the action outputs when you assign
bindings to it. More on that later.

To implement the trait, you can use the provided derive macro.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Jump;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;
```

We also require [`Debug`], which we use for logging. The `output` is the only required parameter.
you can provide additional parameters, see the [`InputAction`] documentation.

## Defining input contexts

Input contexts are a collection of actions that represents a certain context that the player can be in,
like "In car" or "On foot". They describe the rules for what triggers a given action. Contexts can be
dynamically added, removed, or prioritized for each user. Depending on your type of game, you may have
a single global context or multiple contexts for different gameplay states.

Input contexts are represented by unit structs that implement the [`InputContext`] trait. You can use the provided derive
macro for this. Unlike actions, all contexts need to be registered in the app using [`InputContextAppExt::add_input_context`].

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut app = App::new();
# app.add_plugins(EnhancedInputPlugin);
app.add_input_context::<OnFoot>();

#[derive(InputContext)]
struct OnFoot;
```

You can provide additional parameters, see the [`InputContext`] documentation.

## Binding actions

Actions must be bound to inputs such as gamepad or keyboard. While input contexts are defined statically at compile
time, bindings must be assigned at runtime. They're stored inside the [`Actions<C>`] component, where `C` is an input
context. Contexts becomes active only when its component exists on an entity. You can attach multiple [`Actions<C>`]
components to a single entity.

To bind actions, use [`Actions::bind`] followed by one or more [`ActionBinding::to`] calls to define inputs. You can pass any
input type that implements [`IntoBindings`], including tuples for multiple inputs (similar to [`App::add_systems`] in Bevy).
All assigned inputs will be treated as "any of". See [`ActionBinding::to`] for details.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
let mut actions = Actions::<OnFoot>::default();
// The action will trigger when space or gamepad south button is pressed.
actions.bind::<Jump>().to((KeyCode::Space, GamepadButton::South));
# #[derive(InputContext)]
# struct OnFoot;
# #[derive(Debug, InputAction)]
# #[input_action(output = bool)]
# struct Jump;
```

### Input modifiers

Actions know nothing about input sources and simply convert raw values into the [`InputAction::Output`]. Input is read as the
[`ActionValue`] enum, with the variant depending on the input source. For example, key inputs are captured as [`bool`], but
if your action’s output type is [`Vec2`], the value will be assigned to the X axis. See the [`Input`] documentation for how each
source is captured and [`ActionValue::convert`] for details about how values are converted.

However, you might want to apply preprocessing first. For example, invert values, apply sensitivity, or remap axes. This is
where [input modifiers](crate::input_modifier) come in. They represented as structs that implement the [`InputModifier`] trait.
You can attach modifiers to inputs using [`BindingBuilder::with_modifiers`]. Thanks to traits, this works with any input type.
You can also attach modifiers globally via [`ActionBinding::with_modifiers`], which applies to all inputs. For details about
how multiple modifiers are merged together, see the [`ActionBinding`] documentation. Both methods also support tuple syntax
for assigning multiple modifiers at once.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut actions = Actions::<OnFoot>::default();
actions
    .bind::<Move>()
    .to((
        // Keyboard keys captured as `bool`, but the output of `Move` is defined as `Vec2`,
        // so you need to assign keys to axes using swizzle to reorder them and negation.
        KeyCode::KeyW.with_modifiers(SwizzleAxis::YXZ),
        KeyCode::KeyA.with_modifiers(Negate::all()),
        KeyCode::KeyS.with_modifiers((Negate::all(), SwizzleAxis::YXZ)),
        KeyCode::KeyD,
        // In Bevy sticks split by axes and captured as 1-dimensional inputs,
        // so Y stick needs to be sweezled into Y axis.
        GamepadAxis::LeftStickX,
        GamepadAxis::LeftStickY.with_modifiers(SwizzleAxis::YXZ),
    ))
    .with_modifiers((
        // Modifiers applied at the action level.
        DeadZone::default(),    // Normalizes movement.
        SmoothNudge::default(), // Smoothes movement.
    ));
# #[derive(InputContext)]
# struct OnFoot;
# #[derive(Debug, InputAction)]
# #[input_action(output = Vec2)]
# struct Move;
```

You can also attach modifiers to input tuples using [`IntoBindings::with_modifiers_each`]. It works similarly to
[`IntoScheduleConfigs::distributive_run_if`] in Bevy.

### Presets

Some bindings are very common. It would be inconvenient to bind WASD and sticks like in the example above every time.
To solve this, we provide [presets](crate::preset) - structs that store bindings and apply predefined modifiers.
They implement [`IntoBindings`], so you can pass them directly into [`ActionBinding::to`].

For example, you can use [`Cardinal`] and [`Axial`] presets to simplify the example above.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut actions = Actions::<OnFoot>::default();
// We provide a `with_wasd` method for quick prototyping, but you can
// construct this struct with any input.
actions
    .bind::<Move>()
    .to((Cardinal::wasd_keys(), Axial::left_stick()))
    .with_modifiers((DeadZone::default(), SmoothNudge::default()));
# #[derive(InputContext)]
# struct OnFoot;
# #[derive(Debug, InputAction)]
# #[input_action(output = Vec2)]
# struct Move;
```

### Input conditions

Instead of hardcoded states like "pressed" or "released", all actions internally have an abstract [`ActionState`].
Its meaning depends on assigned [input conditions](crate::input_condition), which decide when the action triggers.
This allows to define flexible conditions, such as "hold for 1 second".

Input conditions are structs that implement [`InputCondition`]. Similar to modifiers, you can use
[`BindingBuilder::with_conditions`] for per-input conditions or [`ActionBinding::with_conditions`]
to define a condition that applies to all action's inputs. Conditions are evaluated after input modifiers.
For details about how multiple conditions are merged together, see the [`ActionBinding`] documentation.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut actions = Actions::<OnFoot>::default();
// The action will trigger only if held for 1 second.
actions.bind::<Jump>().to(KeyCode::Space.with_conditions(Hold::new(1.0)));
# #[derive(InputContext)]
# struct OnFoot;
# #[derive(Debug, InputAction)]
# #[input_action(output = bool)]
# struct Jump;
```

If no conditions are assigned, the action will be triggered by any non-zero value.

Similar to modifiers, you can also attach conditions to input tuples using [`IntoBindings::with_conditions_each`].

### Organizing bindings

It's convenient to define bindings in a single function that used every time you activate the context
or reload your application settings.

To achieve this, we provide a special [`Bind<C>`] event that triggers when you insert or replace
[`Actions<C>`] component. Just create an observer for it and define all your bindings there:

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut app = App::new();
app.add_observer(bind_actions);

/// Setups bindings for [`OnFoot`] context from application settings.
fn bind_actions(
    trigger: Trigger<Bind<OnFoot>>,
    settings: Res<AppSettings>,
    mut actions: Query<&mut Actions<OnFoot>>
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Jump>()
        .to((settings.keyboard.jump, GamepadButton::South));
}

#[derive(Resource)]
struct AppSettings {
    keyboard: KeyboardSettings,
}

struct KeyboardSettings {
    jump: KeyCode,
}
# #[derive(InputContext)]
# struct OnFoot;
# #[derive(Debug, InputAction)]
# #[input_action(output = bool)]
# struct Jump;
```

We also provide a user-triggerable [`RebindAll`] event that resets bindings for all inserted
[`Actions`] and also triggers [`Bind`] event for them.

## Reacting on actions

Up to this point, we've only defined actions and contexts but haven't reacted to them yet.
We provide both push-style (via observers) and pull-style (by checking components) APIs.

### Push-style

It's recommended to always use the observer API when possible. Don’t worry about losing parallelism - running a system
has its own overhead, so for small logic, it’s actually faster to execute it outside a system. Just avoid heavy logic in
action observers.

After each input processing cycle, we calculate a new [`ActionState`] and trigger events based on state transition
(including transition between identical states). For details about transition logic and event types, see the [`ActionEvents`]
documentation.

Triggered events are stored as [`ActionEvents`] bitset, but triggered using dedicated types that correspond to bitflags -
[`Started<A>`], [`Fired<A>`], etc., where `A` is your action type. The trigger target will be the entity with the [`Actions`] component
and the output type will match the action’s [`InputAction::Output`]. Events also store other information, such as timings. See the
documentation on specific event type for more information.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut app = App::new();
app.add_observer(apply_movement);

/// Apply movement when `Move` action considered fired.
fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    // Read transform from the context entity.
    let mut transform = players.get_mut(trigger.target()).unwrap();

    // We defined the output of `Move` as `Vec2`,
    // but since translation expects `Vec3`, we extend it to 3 axes.
    transform.translation += trigger.value.extend(0.0);
}
# #[derive(Debug, InputAction)]
# #[input_action(output = Vec2)]
# struct Move;
```

The event system is highly flexible. For example, you can use the [`Hold`] condition for an attack action, triggering strong attacks on
[`Completed`] events and regular attacks on [`Canceled`] events.

### Pull-style

You can also query for [`Actions`] within a system. Use [`Actions::get<A>`] to retrieve an [`Action`], from which you can obtain the
current value, state, or triggered events for this tick as [`ActionEvents`] bitset.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
/// Apply movemenet when `Move` action considered fired.
fn system(players: Single<(&Actions<OnFoot>, &mut Transform)>) -> Result<()> {
    let (actions, mut transform) = players.into_inner();
    if actions.state::<Jump>()? == ActionState::Fired {
        // Apply logic...
    }
#   Ok(())
}
# #[derive(InputContext)]
# struct OnFoot;
# #[derive(Debug, InputAction)]
# #[input_action(output = bool)]
# struct Jump;
```

# Input and UI

Currently, Bevy doesn't have focus management or navigation APIs. But we provide [`ActionSources`] resource
that could be used to prevents actions from triggering during UI interactions. See its docs for details.

# Troubleshooting

If you face any issue, try to enable logging to see what is going on.
To enable logging, you can temporarily set `RUST_LOG` environment variable to `bevy_enhanced_input=debug`
(or `bevy_enhanced_input=trace` for more noisy output) like this:

```bash
RUST_LOG=bevy_enhanced_input=debug cargo run
```

The exact method depends on the OS shell.

Alternatively you can configure `LogPlugin` to make it permanent.
*/

#![no_std]

extern crate alloc;

// Required for the derive macro to work within the crate.
extern crate self as bevy_enhanced_input;

pub mod action_binding;
pub mod action_value;
pub mod actions;
pub mod events;
pub mod input;
pub mod input_action;
pub mod input_binding;
pub mod input_condition;
pub mod input_context;
pub mod input_modifier;
pub mod input_reader;
pub mod input_time;
pub mod preset;
mod trigger_tracker;

pub mod prelude {
    pub use super::{
        EnhancedInputPlugin, EnhancedInputSystem,
        action_binding::{ActionBinding, MockSpan},
        action_value::{ActionValue, ActionValueDim},
        actions::Actions,
        events::*,
        input::{GamepadDevice, Input, InputModKeys, ModKeys},
        input_action::{Accumulation, Action, ActionState, InputAction},
        input_binding::{BindingBuilder, InputBinding, IntoBindings},
        input_condition::{
            ConditionKind, InputCondition, block_by::*, chord::*, down::*, hold::*,
            hold_and_release::*, press::*, pulse::*, release::*, tap::*,
        },
        input_context::{Bind, InputContext, InputContextAppExt, RebindAll},
        input_modifier::{
            InputModifier, accumulate_by::*, clamp::*, dead_zone::*, delta_scale::*,
            exponential_curve::*, negate::*, scale::*, smooth_nudge::*, swizzle_axis::*,
        },
        input_reader::ActionSources,
        input_time::{InputTime, TimeKind},
        preset::*,
    };
    pub use bevy_enhanced_input_macros::{InputAction, InputContext};
}

use bevy::{input::InputSystem, prelude::*};

use input_context::ContextRegistry;
use input_reader::{ActionSources, ResetInput};

#[cfg(doc)]
use prelude::*;

/// Initializes contexts and feeds inputs to them.
///
/// See also [`EnhancedInputSystem`].
pub struct EnhancedInputPlugin;

impl Plugin for EnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextRegistry>()
            .init_resource::<ResetInput>()
            .init_resource::<ActionSources>()
            .configure_sets(PreUpdate, EnhancedInputSystem.after(InputSystem));
    }

    fn finish(&self, app: &mut App) {
        let registry = app
            .world_mut()
            .remove_resource::<ContextRegistry>()
            .expect("registry should be inserted in `build`");

        for contexts in registry.iter() {
            contexts.setup(app);
        }
    }
}

/// Label for the system that updates input context instances.
///
/// Runs in each registered [`InputContext::Schedule`].
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct EnhancedInputSystem;
