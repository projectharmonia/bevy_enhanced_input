# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Logging for binding.
- `ActionsData::insert_action`.
- `BlockBy::events` to block only events. Could be used for chords to avoid triggering required actions.

### Changed

- Rework `ConditionKind` and the logic around it to make the behavior close to UE.
- Remove world access from conditions and modifiers. This means that you no longer can write game-specific conditions or modifiers. But it's much nicer (and faster) to just do it in observers instead.
- Values from `Input` are now converted to the action-level dimension only after applying all input-level modifiers and conditions. This allows things like mapping the Y-axis of `ActionValue::Axis2D` into an action with `ActionValueDim::Axis1D`.
- Rename `ActionBind::with_axis2d` into `ActionBind::with_xy_axis`.
- Rename `ScaleByDelta` into `DeltaScale`.
- Rename `Released` into `Release`.
- Rename `Pressed` into `Press`.
- Rename `BlockedBy` into `BlockBy`.
- Rename `Scalar` into `Scale`.
- Split `ActionData::update` into `ActionData::update_time`, `ActionData::trigger_events` and `ActionData::set_state`.
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

## [0.1.0] - 2024-10-20

Initial release.

[unreleased]: https://github.com/projectharmonia/bevy_replicon/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/projectharmonia/bevy_replicon/releases/tag/v0.1.0
