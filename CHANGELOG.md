# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Logging for binding.

### Changed

- Replace `with_held_timer` with `relative_speed` that just accepts a boolean.
- Rename `HeldTimer` into `ConditionTimer`.
- Use Use `trace!` instead of `debug!` for triggered events.

## [0.1.0] - 2024-10-20

Initial release.

[unreleased]: https://github.com/projectharmonia/bevy_replicon/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/projectharmonia/bevy_replicon/releases/tag/v0.1.0
