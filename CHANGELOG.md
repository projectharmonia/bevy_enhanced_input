# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- `Action<A>` now implements `Clone` and `Copy` for any `A`.
- `ActionEvents` now implements `Serialize` and `Deserialize`.
- Split `EnhancedInputSystem` into `EnhancedInputSet::Update` (reads new inputs from the `InputReader` and updates the `Actions` components) and `EnhancedInputSet::Trigger` (triggers the events corresponding to how the `Actions` components changed).
- Move most of the `Actions<C>` functionality to an untyped struct `UntypedActions`. `Actions<C>` derefs to `UntypedActions`, so you don't have to change any call sites.

## [0.13.0] - 2025-06-19

### Added

- `Actions::mock` and related methods to mock actions.
- Add duplicative swizzles to `SwizzleAxis.`
- `Actions::bindings` to get bindings access.
- `Actions::iter` and `Actions::iter_mut` to read and write actions data.

### Changed

- Rename `Binding` event into `Bind`.
- Rename `RebuildBindings` event into `RebindAll`.
- Rename `action_instance` module into `input_context` and move `InputContext` to it.
- Conditions and modifiers now accept the newly added `InputTime` system parameter, which dereferences to `Time`. From it, you can also access `Time<Real>` if you need time that is not affected by time dilation.
- Rename `relative_speed` into `with_time_kind` and accept the newly added `TimeKind` enum instead of boolean.
- All conditions with timer no longer implement `Copy`.
- Rename `input_condition::press` into `input_condition::down` and `input_condition::just_press` into `input_condition::press`. Their structs were renamed in the previous release, but the modules weren't.
- Merge `acton_map` module into `input_action`.
- Move `action_binding`, `actions`, `events`, `input_action`, `input_binding`, `input_condition`, `input_modifier` and `preset` modules under `input_context` module.
- Make all data fields of `Action` public.
- Return the strongly typed output of an action from `Actions::value`, similar to triggers.
- Rename `ActionOutput::as_output` into `ActionOutput::unwrap_value`.
- Rename `Action` into `UntypedAction`.
- `Actions::get` now returns a typed `Action<A>`.

### Removed

- `BlockBy::events_only` and `ConditionKind::Blocker::events_only`. This functionality was added before the introduction of the pull-based API and caused inconsistencies in returned values. It was intended to be used with `Chord`. If you need an action to be part of a chord but only want to react to it when the chord is not active, just check its state in the observer.
- `ActionMap::insert`. Use the new action mocking API.
- `Action::new`, `Action::trigger_events` and `Action::update` from the public API. Use the new action mocking API.
- `ConditionTimer`. Use Bevy's `Timer`. Use `InputTime::delta_kind` if you need a configurable time dilation.
- `ActionMap`. Use Bevy's `TypeIdMap` instead.
- Getters for `Action`. Use the fields, which are now public.

## [0.12.0] - 2025-05-25

### Added

- `Actions::state`, `Actions::value` and `Actions::events` helpers to obtain specific information for an action directly.
- `ConditionTimer::with_duration`.

### Changed

- Rename `Press` into `Down`.
- Rename `JustPress` into `Press`.
- Return `Result` from `Actions::bindings` to integrate with Bevy's unified error handling system.
- Rename `Actions::action` into `Actions::get` and return `Result`.
- Fix dimension cutoff when using `SwizzleAxis` with `Axis2D`.

### Removed

- `Actions::get_action`. Use `Actions::get`.
- `Actions::get_binding`. Use `Actions::binding`.

## [0.11.0] - 2025-04-24

### Added

- `InputContext::Schedule` to control the schedule in which the context will be evaluated.

### Changed

- Update to Bevy 0.16.

## [0.10.0] - 2025-04-24

### Added

- `Clamp` modifier.
- Serde derives for `ActionState`.
- `Axial` preset to map any axis into 2-dimensional input.

### Removed

- `GamepadStick`. Use `Axial::left_stick()` or `Axial::right_stick()` instead.

## [0.9.0] - 2025-04-08

### Added

- `ActionSources` resource to control which input sources are visible to actions.

### Changed

- Rename `InputContext` into `Actions<C>` with its module. Now it's a component. Since a single entity could have multiple actions, the struct now have associated marker `C`. This marker needs to implement the newly added `InputContext` trait. We provide a derive macro for it.
- Rather than auto-inserting `Actions` when specified input context component is added, users now insert `Actions` directly. The type passed into `InputContextAppExt::add_input_context` is now just a marker for `Actions`, not necessarily a component.
- Previously users needed to insert component that was registered as input context. Now context is just a regular struct and users need to insert `Actions<C>` directly.
- Input contexts no longer associated with a component. Users need to manually insert `Actions`.
- `Binding` trigger no longer stores bindings. Just get `Actions` using `Query` and mutate it directly.
- Rename `ActionBind` into `ActionBinding` and move into `action_binding` module.
- Rename `ActionBinding::bindings` into `ActionBindings::inputs`.
- Rename `Actions::action_bind` and `Actions::get_action_bind` into `Actions::binding` and `Actions::get_binding`.
- Rename `input_bind::InputBind` into `input_binding::InputBinding`.
- Rename `InputBindSet` into `IntoBindings` and its `bindings` method into `into_bindings`.
- Rename `InputConditionSet` into `IntoConditions` and its `conditions` method into `into_conditions`.
- Rename `InputModifierSet` into `IntoModifiers` and its `modifiers` method into `into_modifiers`.
- Rename `InputBindModifierEach` and `InputBindConditionEach` into `WithModifiersEach` and `WithConditionsEach` respectively.
- Rename `InputBindModCond` into `BindingBuilder`.
- Rename `ActionsData` into `ActionMap`.
- Rename `ActionData` into `Action`.
- Move `ActionMap`, `Action` and `ActionState` into `action_map` module.

### Removed

- `InputContextRegistry`. Directly access `Actions` from entities.
- `ui_priority` and `egui_priority` features. Use `ActionSources` manually for your UI library of choice.

### Fixed

- Crash on context switch due to unpredictable observers ordering.

## [0.8.0] - 2025-03-17

### Added

- Implement `Display` for `Input` and `ModKeys`.
- `ModKeys::pressed` to get currently active modifiers.

### Changed

- Rename `ContextInstance` into `InputContext`. `InputContext` is no longer a trait. Bindings now configured via observer on `Bindings<C>` (see docs on `InputContextAppExt::add_input_context` for details). Priority now set at runtime via `InputContext::set_priority`.
- Move all child modules from `input_context` under crate root (one level upper).
- Capture gamepad buttons as `Axis1D` because triggers are also buttons and sometimes have analog value.
- Rename `input_context` module into `registry` and `context_instance` into `input_context`.
- Rename `ContextAppExt` into `InputContextAppExt`.
- Rename `ContextInstances` into `InputContextRegistry`.
- Rename `RebuildInputContexts` into `RebuildBindings`.
- Reorder constants in `ModeKeys`.
- Derive `Reflect` for `Input` and `ModKeys`.
- Derive `PartialEq` for `Input`.

### Fixed

- Reversed `west` and `east` directions for `Cardinal`.

## [0.7.2] - 2025-01-24

### Changed

- Update documentation.

## [0.7.1] - 2025-01-21

### Added

- `ContextInstance::get_action_bind` and `ContextInstance::action_bind` to access action bindings.
- `ActionBind::bindings` to access input bindings.

## [0.7.0] - 2025-01-19

### Added

- `InputAction::REQUIRE_RESET` switch to require inputs to be zero before the first activation and continue to consume them even after context removal until inputs become zero again. Previously, this behavior was always enabled but applied only to the first activation. Now it’s disabled by default and also applies to removals. I believe this new default behavior is more intuitive.

### Changed

- Snap to the target in `SmoothNudge` when the current value is close enough.
- Use `debug!` instead of `trace!` and print action name for events logging.

## [0.6.0] - 2025-01-13

### Changed

- Replace `SmoothDelta` modifier with `SmoothNudge`. It uses `StableInterpolate::smooth_nudge`, which properly interpolates inputs across frames.
- Rename `InputContext::get` into `InputContext::get_context`. We now provide `InputContext::context` for panicking version. Handling failing case is unlikely needed and most of the time users know that the context exists.
- Rename `ContextInstance::action` into `ContextInstance::get_action`. We now provide `ContextInstance::action` for panicking version.

### Removed

- `InputContext::MODE`. All contexts now work like `InputContext::Exclusive` (the default). If you want to share the same input across multiple entities, use a separate "controller" entity for a context and apply inputs to the desired entities. This approach is more efficient and explicit, so it's not worth having a special case in the crate for it.

## [0.5.0] - 2024-12-05

### Added

- `InputBindSet::with_modifiers_each` and `InputBindSet::with_conditions_each` to add conditions and modifiers to sets.

### Changed

- `Negate`'s functions `x`, `y`, `z` no longer take an `invert` parameter and assume it is `true`.
- `Negate::all`'s current function has been moved to `Negate::splat`.
- `Negate::all` no longer takes an `invert` parameter and assumes it is `true` (opposite is `Negate::none`).
- Rename `Biderectional` to `Bidirectional` to fix typo.
- Rename `InputConditions` to `InputConditionSet`.
- Rename `InputConditionSet::iter_conditions` to `InputConditionSet::conditions`.
- Rename `InputModifiers` to `InputModifierSet`.
- Rename `InputModifierSet::iter_modifiers` to `InputModifierSet::modifiers`.
- Rename `InputBindings` to `InputBindSet`.
- Rename `InputBindSet::iter_bindings` to `InputBindSet::bindings`.

### Removed

- `Negate::default` - use `Negate::all` instead.

## [0.4.0] - 2024-12-01

### Changed

- Update to Bevy 0.15.

## [0.3.0] - 2024-12-01

### Changed

- Replace `InputAction::DIM` with `InputAction::Output` where you set the type directly (`bool`, `f32`, `Vec2` or `Vec3`) instead of the `ActionValueDim`.
- `InputAction` macro now accepts `output = <Type>` instead of `dim = <ActionValueDim variant>`.
- All events now have typed values, based on `InputAction::Output`.
- Rename `ActionBind::with` into `ActionBind::to`.
- All presets now structs (see `input_context::context_instance::input_preset`) that can be passed directly into `ActionBind::to`.
- Rename `Modifiers` into `ModKeys` to avoid confusion with input modifiers.
- Rename all `modifiers` fields in the `Input` enum into `mod_keys`.
- Rename `Input::with_modifiers` and `Input::without_modifiers` into `Input::with_mod_keys` and `Input::without_mod_keys`, respectively.
- `Input::with_mod_keys` now a trait method which implemented for any type that can be converted into an input to ergonomically assign keyboard modifiers.
- `Input::without_mod_keys` no longer `const`.
- Rename all `with_modifier` and `with_condition` methods into `with_modifiers` and `with_conditions` (plural) because now they accept tuples.
- `InputBind::with_modifiers` and `InputBind::with_conditions` now trait methods which implemented for any type that can be converted into a binding to ergonomically assign modifiers and conditions.
- Move `ActionState`, `ActionData` and `ActionsData` to `context_instance` module.
- Move `InputBind` to newly created `input_context::input_bind` module.

### Fixed

- Macro hygiene.

## [0.2.1] - 2024-11-20

### Fixed

- Correctly handle generics in the derive macro.

### Changed

- Rename `Press` into `JustPress`.
- Rename `Down` into `Press` to avoid collision with `Down` from `bevy_picking`.
- Replace `ContextInstance::with_gamepad` builder with just `ContextInstance::set_gamepad` setter.

## [0.2.0] - 2024-11-03

### Added

- Logging for binding.
- `AccumulateBy` modifier.
- `ActionsData::insert_action` to insert a data for `A`.
- `ActionData::events` to get triggered events from the last update.
- `ActionData::value` to get triggered value from the last update.
- `ActionData::trigger_events` to trigger events based on the last `ActionData::update`.
- `BlockBy::events` to block only events. Could be used for chords to avoid triggering required actions.
- `Deref` for `ActionEvent::kind`.
- `ContextInstances` to public API and methods to get `ActionData` for an action.

### Changed

- All events now separate structs instead of enum.
- Modifiers now accept `ActionsData`.
- Rework `ConditionKind` and the logic around it to make the behavior close to Unreal Engine.
- Consume inputs only if the action state is not equal to `ActionState::None`.
- Remove world access from conditions and modifiers. This means that you no longer can write game-specific conditions or modifiers. But it's much nicer (and faster) to just do it in observers instead.
- Values from `Input` are now converted to the action-level dimension only after applying all input-level modifiers and conditions. This allows things like mapping the Y-axis of `ActionValue::Axis2D` into an action with `ActionValueDim::Axis1D`.
- Rename `ActionBind::with_axis2d` into `ActionBind::with_xy_axis`.
- Rename `ScaleByDelta` into `DeltaScale`.
- Rename `Released` into `Release`.
- Rename `Pressed` into `Press`.
- Rename `BlockedBy` into `BlockBy`.
- Rename `Scalar` into `Scale`.
- `ActionData::update` now accepts a value and no longer trigger events.
- Use `isize` for `InputContext::PRIORITY`.
- Replace `SmoothDelta` with `LerpDelta` that does only linear interpolation. Using easing functions for inputs doesn't make much sense.
- Modifiers are now allowed to change passed value dimensions.
- All built-in modifiers now handle values of any dimension.
- Replace `with_held_timer` with `relative_speed` that just accepts a boolean.
- Rename `HeldTimer` into `ConditionTimer`.
- Use `trace!` instead of `debug!` for triggered events.

### Removed

- `ignore_incompatible!` since no longer needed.
- `SwizzleAxis::XXX`, `SwizzleAxis::YYY` and `SwizzleAxis::ZZZ`. They encourage a bad pattern of defining actions with duplicate data. Duplicate axes inside the trigger if needed.
- `ActionData::trigger_removed`, use `ActionData::trigger_events` instead.
- `Normalize` modifier, use `DeadZone::default` instead to properly work with analogue inputs.

## [0.1.0] - 2024-10-20

Initial release.

[unreleased]: https://github.com/projectharmonia/bevy_replicon/compare/v0.13.0...HEAD
[0.13.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/projectharmonia/bevy_replicon/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/projectharmonia/bevy_replicon/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/projectharmonia/bevy_replicon/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/projectharmonia/bevy_replicon/releases/tag/v0.1.0
