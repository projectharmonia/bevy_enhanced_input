# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- `Negate`'s functions `x`, `y`, `z` no longer take an `invert` parameter and assume it is `true`.
- `Negate::all`'s current function has been moved to `Negate::splat`.
- `Negate::all` no longer takes an `invert` parameter and assumes it is `true` (opposite is `Negate::none`).

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
- Rename `Modifiers` into `ModKeys` to avod confusion with input modifiers.
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
- Rework `ConditionKind` and the logic around it to make the behavior close to UE.
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
- All built-in modifiers now handle values of any dimention.
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

[unreleased]: https://github.com/projectharmonia/bevy_replicon/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/projectharmonia/bevy_replicon/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/projectharmonia/bevy_replicon/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/projectharmonia/bevy_replicon/releases/tag/v0.1.0
