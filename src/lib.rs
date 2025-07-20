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

## Input contexts

Contexts are regular components that represent a certain input state the player can be in,
like "In Car" or "On Foot". Depending on your type of game, you may have a single global context
or multiple contexts for different gameplay states.

All contexts need to be registered in the app using [`InputContextAppExt::add_input_context`].

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut app = App::new();
# app.add_plugins(EnhancedInputPlugin);
app.add_input_context::<OnFoot>();

#[derive(Component)]
struct OnFoot;
```

By default, contexts evaluated during [`PreUpdate`], but you can override this using
[`InputContextAppExt::add_input_context_to`]. For example, if your game logic runs inside
[`FixedMain`](bevy::app::FixedMain), you should set the schedule to [`FixedPreUpdate`].

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut app = App::new();
# app.add_plugins(EnhancedInputPlugin);
app.add_input_context_to::<FixedPreUpdate, Player>();

#[derive(Component)]
struct Player;
```

## Input Actions

Actions represent something the user can do, like "Crouch" or "Fire Weapon". They are represented
by the [`Action<A>`] component, where `A` is a user-defined marker that implements the [`InputAction`] trait.
Each action has an associated [`InputAction::Output`] type - this is the value the action produces
when you assign bindings to it. More on that later.

To implement the trait, you can use the provided derive macro.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;
```

## Spawning

Contexts can be associated with actions using the [`ActionOf`] relationship. We provide the [`actions!`] macro,
which is similar to [`related!`], but instead of specifying [`Actions<C>`], you only write `C` itself.
The relationship is generic over `C` because a single entity can have multiple associated contexts.
Each item should be a bundle that will be spawned as its own entity.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
// Spawn an entity with `OnFoot` context component and actions with `ActionOf<OnFoot>` relationship.
world.spawn((
    OnFoot,
    actions!(OnFoot[
        Action::<Jump>::new(),
        Action::<Fire>::new(),
    ])
));
# #[derive(Component)]
# struct OnFoot;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Fire;
```

## Bindings

Actions need to be bound to inputs, such as a gamepad or keyboard. These bindings are represented by the [`Binding`]
which can be constructed from various input types. Bindings can be associated with actions using the [`BindingOf`]
relationship. Similar to [`actions!`], we provide the [`binding!`] macro to spawn related bindings. But unlike
[`ActionOf<C>`], it's not generic, since each action is represented by a separate entity. Items can either be individual
values that implement [`Into<Binding>`], or tuples where the first element implements [`Into<Binding>`] and the remaining
elements are regular components or bundles.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
world.spawn((
    OnFoot,
    actions!(OnFoot[
        (
            // Spawn an entity with `Action<Jump>` action component and bindings with `BindingOf` relationship.
            Action::<Jump>::new(),
            bindings![KeyCode::Space, GamepadButton::South],
        ),
        (
            Action::<Fire>::new(),
            bindings![MouseButton::Left, GamepadButton::RightTrigger2],
        ),
    ])
));
# #[derive(Component)]
# struct OnFoot;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Fire;
```

## Input modifiers

Action values are stored inside the [`Action<C>`] component in a typed form, and in the [`ActionValue`] component in a dynamically
typed form (which is one of the required components of [`Action<C>`]).

During [`EnhancedInputSet::Update`], we read input for each [`Binding`] as an [`ActionValue`] (the variant depends on
the input source) and convert it to the [`ActionValue`] on the associated action entity. For example, key inputs are captured
as [`bool`], but if your action’s output type is [`Vec2`], the value will be assigned to the X axis as `0.0` or `1.0`.
See the [`Binding`] documentation for how each source is captured, and [`ActionValue::convert`] for details on how
values are converted. It's very straightforward,

Then, during [`EnhancedInputSet::Apply`], the value from [`ActionValue`] is written into [`Action<C>`].

However, you might want to apply preprocessing first - for example, invert values, apply sensitivity, or remap axes. This is
where [input modifiers](crate::modifier) come in. They are components that implement the [`InputModifier`] trait and can
be attached to both actions and bindings. Binding-level modifiers are applied first, followed by action-level modifiers.
Use action-level modifiers as global modifiers that are applied to all bindings of the action.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
world.spawn((
    OnFoot,
    actions!(OnFoot[
        (
            Action::<Move>::new(),
            // Modifier components at the action level.
            DeadZone::default(),    // Applies non-uniform normalization.
            SmoothNudge::default(), // Smoothes movement.
            bindings![
                // Keyboard keys captured as `bool`, but the output of `Move` is defined as `Vec2`,
                // so you need to assign keys to axes using swizzle to reorder them and negation.
                (KeyCode::KeyW, SwizzleAxis::YXZ),
                (KeyCode::KeyA, Negate::all()),
                (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
                (KeyCode::KeyD),
                // In Bevy sticks split by axes and captured as 1-dimensional inputs,
                // so Y stick needs to be sweezled into Y axis.
                (GamepadAxis::LeftStickX),
                (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
            ]
        ),
    ]),
));
# #[derive(Component)]
# struct OnFoot;
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Move;
```

### Presets

Some bindings are very common. It would be inconvenient to bind WASD keys and analog sticks manually, like in the example above,
every time. To solve this, we provide [presets](crate::preset) - structs that implement [`SpawnableList`] and store bindings that
will be spawned with predefined modifiers.

Due to how relationship spawning works in Bevy, you can't use [`SpawnableList`]s with the macro; instead, you need to pass them
to [`Bindings::spawn`], similar to how you would use [`Children::spawn`] when working with [`SpawnWith`](bevy::ecs::spawn::SpawnWith)
or any other struct that implements [`SpawnableList`].

For example, you can use [`Cardinal`] and [`Axial`] presets to simplify the example above.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
world.spawn((
    OnFoot,
    actions!(OnFoot[
        (
            Action::<Move>::new(),
            DeadZone::default(),
            SmoothNudge::default(),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Axial::left_stick(),
                // You can also pass additional bindings here, but you'll need to
                // use `Binding::from`, which the macro previously handled for you.
            )),
        ),
    ]),
));
# #[derive(Component)]
# struct OnFoot;
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Move;
```

You can assign custom bindings or attach additional components by manually initializing the preset fields. You can pass anything that
implements [`Bundle`]. Each built-in preset also implements [`WithBundle`] trait to conveniently attach components to every field
using [`WithBundle::with`].

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
Bindings::spawn((
    Cardinal {
        north: Binding::from(KeyCode::KeyI),
        east: Binding::from(KeyCode::KeyL),
        south: Binding::from(KeyCode::KeyK),
        west: Binding::from(KeyCode::KeyJ),
    },
    Axial::left_stick().with((Scale::splat(1.0), SmoothNudge::default())),
));
```

## Input conditions

Instead of hardcoded states like "pressed" or "released", all actions use an abstract [`ActionState`] component
(which is a required component of [`Action<C>`]). Its meaning depends on the assigned [input conditions](crate::condition),
which determine when the action is triggered. This allows you to define flexible behaviors, such as "hold for 1 second".

Input conditions are components that implement [`InputCondition`] trait. Similar to modifiers, you can attach them to
both actions and bindings. They also evaluated during [`EnhancedInputSet::Update`] right after modifiers and update
[`ActionState`] on the associated action entity.

If no conditions are attached, the action behaves like with [`Down`] condition with a zero actuation threshold,
meaning it will trigger on any non-zero input value.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
world.spawn((
    OnFoot,
    actions!(OnFoot[
        (
            // The action will trigger only if held for 1 second.
            Action::<Jump>::new(),
            Hold::new(1.0),
            bindings![KeyCode::Space, GamepadButton::South],
        ),
        (
            Action::<Fire>::new(),
            Pulse::new(0.5), // The action will trigger every 0.5 seconds while held.
            bindings![
                (GamepadButton::RightTrigger2, Down::new(0.3)), // Additionally the right trigger only counts if its value is greater than 0.3.
                (MouseButton::Left),
            ]
        ),
    ])
));
# #[derive(Component)]
# struct OnFoot;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Fire;
```

## Mocking

You can also mock an action value using the [`ActionMock`] component. Simply insert it into the action with the desired values, and it will drive
the [`ActionState`] and [`ActionValue`] for the specified [`MockSpan`] duration. During this time, all bindings for this action will be ignored.
For more details, see the [`ActionMock`] documentation.

## Evaluation

Context hierarchies will be evaluated in the schedule associated at context registration. Contexts registered in the same
schedule will be evaluated in their spawning order. But you can override this with [`ContextPriority`] component. By default
the priority is 0. Contexts with higher priority evaluated first.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
world.spawn((
    OnFoot,
    ContextPriority::<OnFoot>::new(1), // `OnFoot` context will be evaluated earlier.
    InCar,
    // Actions...
));
# #[derive(Component)]
# struct OnFoot;
# #[derive(Component)]
# struct InCar;
```

Ordering matters because actions "consume" inputs, making them unavailable to other actions until the context that consumed them
is evaluated again. This enables layered contexts, where some actions replace others. Importantly, this does **not** affect
the underlying Bevy input - only the action evaluation logic is impacted. This behavior can be disabled per-action by setting
[`ActionSettings::consume_input`] to `false`. For more details, see the [`ActionSetting`] component documentation.

Actions are ordered by the maximum number of keyboard modifiers in their bindings. For example, an action with a `Ctrl + C` binding
is evaluated before one with just a `C` binding. If actions have the same modifier count, they are ordered by their spawn order.

Action evaluation follows these steps:

- If the action has an [`ActionMock`] component, use the mocked [`ActionValue`] and [`ActionState`] directly.
- Otherwise, evaluate the action from its bindings:
    1. Iterate over each binding from the [`Bindings`] component.
        1.1 Read the input as an [`ActionValue`], or [`ActionValue::zero`] if the input was already consumed by another action.
          The enum variant depends on the input source.
        1.2 Apply all binding-level [`InputModifier`]s.
        1.3 Evaluate all input-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
    2. Select all [`ActionValue`]s with the most significant [`ActionState`] and combine them using the
       [`ActionSettings::accumulation`] strategy.
    3. Convert the combined value to [`ActionOutput::DIM`] using [`ActionValue::convert`].
    4. Apply all action-level [`InputModifier`]s.
    5. Evaluate all action-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
    6. Convert the final value to [`ActionOutput::DIM`] again using [`ActionValue::convert`].
    7. Apply the resulting [`ActionState`] and [`ActionValue`] to the action entity.
    8. If the final state is not [`ActionState::None`], consume the input value.

This logic may seem complex, but you don’t need to memorize it—its behavior is surprisingly intuitive in practice.

This logic may look complicated, but you don't have to memorize it. It behaves surprisingly intuitively.

## Reacting on actions

Up to this point, we've only defined actions and contexts but haven't reacted to them yet.
We provide both push-style (via observers) and pull-style (by checking components) APIs.

### Push-style

It's recommended to use the observer API, especially for actions that trigger rarely. Don't worry about losing parallelism - running
a system has its own overhead, so for small logic, it's actually faster to execute it outside a system. Just avoid heavy logic in
action observers.

During [`EnhancedInputSet::Apply`], events are triggered based on transitions of [`ActionState`], such as
[`Started<A>`], [`Fired<A>`], and others, where `A` is your action type. This includes transitions between identical states.
For a full list of transition events, see the [`ActionEvents`] component documentation.

The event target will be the entity with the context component, and the output type will match the action's [`InputAction::Output`].
Events also include additional data, such as timings and state. See the documentation for each [event type](crate::action::events)
for more details.

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
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Move;
```

The event system is highly flexible. For example, you can use the [`Hold`] condition for an attack action, triggering strong attacks on
[`Completed`] events and regular attacks on [`Canceled`] events.

### Pull-style

During [`EnhancedInputSet::Apply`], the value from the [`ActionValue`] component is written to the [`Action<C>`] component.

You can simply query [`Action<C>`] in a system to get the action value in a strongly typed form.
Alternatively, you can access [`ActionValue`] in its dynamically typed form, which is primarily intended for use during input evaluation
or when integrating with networking or scripting crates.

To access the action state, use the [`ActionState`] component. State transitions are recorded in the [`ActionEvents`]
component, which lets you detect when an action has just started or stopped triggering. It effectively acts as a bitset
representation of all transition events triggered during the current evaluation.

Timing information provided via [`ActionTime`] component.

You can also use Bevy's change detection on components - they're only marked as changed if their values actually change.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
fn apply_input(
    jump_events: Single<&ActionEvents, With<Action<Jump>>>,
    move_action: Single<&Action<Move>>,
    mut player_transform: Single<&mut Transform, With<Player>>,
) {
    // Jumped this frame
    if jump_events.contains(ActionEvents::STARTED) {
        // User logic...
    }

    // We defined the output of `Move` as `Vec2`,
    // but since translation expects `Vec3`, we extend it to 3 axes.
    player_transform.translation = move_action.extend(0.0);
}
# #[derive(Component)]
# struct Player;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Move;
```

# Input and UI

Currently, we don't integrate `bevy_input_focus` directly. But we provide [`ActionSources`] resource
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

[`SpawnableList`]: bevy::ecs::spawn::SpawnableList
*/

#![no_std]

extern crate alloc;

// Required for the derive macro to work within the crate.
extern crate self as bevy_enhanced_input;

pub mod action;
pub mod binding;
pub mod condition;
pub mod context;
pub mod modifier;
pub mod preset;

pub mod prelude {
    pub use super::{
        EnhancedInputPlugin, EnhancedInputSet,
        action::{
            Accumulation, Action, ActionMock, ActionOutput, ActionSettings, ActionState,
            ActionTime, InputAction, MockSpan,
            events::*,
            relationship::{ActionOf, ActionSpawner, ActionSpawnerCommands, Actions},
            value::{ActionValue, ActionValueDim},
        },
        actions,
        binding::{
            Binding, InputModKeys,
            mod_keys::ModKeys,
            relationship::{BindingOf, BindingSpawner, BindingSpawnerCommands, Bindings},
        },
        bindings,
        condition::{
            ConditionKind, InputCondition, block_by::*, chord::*, down::*,
            fns::InputConditionAppExt, hold::*, hold_and_release::*, press::*, pulse::*,
            release::*, tap::*,
        },
        context::{
            ActionsQuery, ContextPriority, GamepadDevice, InputContextAppExt,
            input_reader::ActionSources,
            time::{ContextTime, TimeKind},
        },
        modifier::{
            InputModifier, accumulate_by::*, clamp::*, dead_zone::*, delta_scale::*,
            exponential_curve::*, fns::InputModifierAppExt, negate::*, scale::*, smooth_nudge::*,
            swizzle_axis::*,
        },
        preset::{WithBundle, axial::*, bidirectional::*, cardinal::*, ordinal::*, spatial::*},
    };
    pub use bevy_enhanced_input_macros::InputAction;
}

use bevy::{input::InputSystem, prelude::*};

use condition::fns::ConditionRegistry;
use context::{
    ContextRegistry,
    input_reader::{self, ConsumedInputs, PendingInputs},
};
use modifier::fns::ModifierRegistry;
use prelude::*;

/// Initializes contexts and feeds inputs to them.
///
/// See also [`EnhancedInputSet`].
pub struct EnhancedInputPlugin;

impl Plugin for EnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextRegistry>()
            .init_resource::<ConsumedInputs>()
            .init_resource::<PendingInputs>()
            .init_resource::<ActionSources>()
            .init_resource::<ConditionRegistry>()
            .init_resource::<ModifierRegistry>()
            .register_type::<ActionValue>()
            .register_type::<ActionState>()
            .register_type::<ActionTime>()
            .register_type::<ActionEvents>()
            .register_type::<ActionSettings>()
            .register_type::<ActionMock>()
            .register_type::<Binding>()
            .register_type::<Bindings>()
            .register_type::<BindingOf>()
            .register_type::<GamepadDevice>()
            .register_type::<BlockBy>()
            .register_type::<Chord>()
            .register_type::<Down>()
            .register_type::<Hold>()
            .register_type::<HoldAndRelease>()
            .register_type::<Press>()
            .register_type::<Pulse>()
            .register_type::<Release>()
            .register_type::<Tap>()
            .register_type::<AccumulateBy>()
            .register_type::<Clamp>()
            .register_type::<DeadZone>()
            .register_type::<DeltaScale>()
            .register_type::<ExponentialCurve>()
            .register_type::<Negate>()
            .register_type::<Scale>()
            .register_type::<SmoothNudge>()
            .register_type::<SwizzleAxis>()
            .add_input_condition::<BlockBy>()
            .add_input_condition::<Chord>()
            .add_input_condition::<Down>()
            .add_input_condition::<Hold>()
            .add_input_condition::<HoldAndRelease>()
            .add_input_condition::<Press>()
            .add_input_condition::<Pulse>()
            .add_input_condition::<Release>()
            .add_input_condition::<Tap>()
            .add_input_modifier::<AccumulateBy>()
            .add_input_modifier::<Clamp>()
            .add_input_modifier::<DeadZone>()
            .add_input_modifier::<DeltaScale>()
            .add_input_modifier::<ExponentialCurve>()
            .add_input_modifier::<Negate>()
            .add_input_modifier::<Scale>()
            .add_input_modifier::<SmoothNudge>()
            .add_input_modifier::<SwizzleAxis>()
            .configure_sets(
                PreUpdate,
                (EnhancedInputSet::Prepare, EnhancedInputSet::Update)
                    .chain()
                    .after(InputSystem),
            )
            .add_observer(action::remove_action)
            .add_systems(
                PreUpdate,
                input_reader::update_pending.in_set(EnhancedInputSet::Prepare),
            );
    }

    fn finish(&self, app: &mut App) {
        let context = app
            .world_mut()
            .remove_resource::<ContextRegistry>()
            .expect("contexts registry should be inserted in `build`");

        let conditions = app
            .world_mut()
            .remove_resource::<ConditionRegistry>()
            .expect("conditions registry should be inserted in `build`");

        let modifiers = app
            .world_mut()
            .remove_resource::<ModifierRegistry>()
            .expect("conditions registry should be inserted in `build`");

        for contexts in &*context {
            contexts.setup(app, &conditions, &modifiers);
        }
    }
}

/// Label for the system that updates input context instances.
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum EnhancedInputSet {
    /// Updates list of pending inputs to ignore.
    ///
    /// Runs in [`PreUpdate`].
    Prepare,
    /// Updates the state of the input contexts from inputs and mocks.
    ///
    /// Runs in each registered [`InputContext::Schedule`].
    Update,
    /// Applies the value from [`ActionValue`] to [`Action`] and triggers
    /// events evaluated from [`Self::Update`].
    ///
    /// Runs in each registered [`InputContext::Schedule`].
    Apply,
}
