# Bevy Enhanced Input

[![crates.io](https://img.shields.io/crates/v/bevy_enhanced_input)](https://crates.io/crates/bevy_enhanced_input)
[![docs.rs](https://docs.rs/bevy_enhanced_input/badge.svg)](https://docs.rs/bevy_enhanced_input)
[![codecov](https://codecov.io/gh/projectharmonia/bevy_enhanced_input/graph/badge.svg?token=wirFEuKmMz)](https://codecov.io/gh/projectharmonia/bevy_enhanced_input)

Dynamic and contextual input mappings for Bevy inspired by [Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine).

## Features

* Map inputs from various sources (keyboard, gamepad, etc.) to gameplay actions like `Jump`, `Move`, or `Attack`.
* Assign actions to different contexts like `OnFoot` or `InCar`, which are regular components.
* Activate or deactivate contexts by simply adding or removing components.
* Control how actions accumulate input from sources and consume it.
* Layer multiple contexts on a single entity, controlled by priority.
* Apply modifiers to inputs, such as dead zones, inversion, scaling, etc., or create custom modifiers by implementing a trait.
* Assign conditions for how and when an action is triggered, like "hold", "double-tap" or "chord", etc. You can also create custom conditions, such as "on the ground".
* React on actions with observers.

## Getting Started

Check out the [quick start guide](https://docs.rs/bevy_enhanced_input) for more details.

See also examples in the repo. [simple.rs](examples/simple.rs) should be a good starting point.

Have any questions? Feel free to ask in the dedicated [`bevy_enhanced_input` channel](https://discord.com/channels/691052431525675048/1297361733886677036) in Bevy's Discord server.

## Bevy compatibility

| bevy        | bevy_enhanced_input |
| ----------- | ------------------- |
| 0.14.0      | 0.1                 |
