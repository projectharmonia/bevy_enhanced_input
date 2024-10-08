# Bevy Enhanced Input

[![crates.io](https://img.shields.io/crates/v/bevy_enhanced_input)](https://crates.io/crates/bevy_enhanced_input)
[![docs.rs](https://docs.rs/bevy_enhanced_input/badge.svg)](https://docs.rs/bevy_enhanced_input)
[![codecov](https://codecov.io/gh/projectharmonia/bevy_enhanced_input/graph/badge.svg?token=wirFEuKmMz)](https://codecov.io/gh/projectharmonia/bevy_enhanced_input)

Dynamic and contextual input mappings inspired by [Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine) for Bevy.

## What makes Enhanced Input... enhanced?

Instead of directly reacting to inputs from various sources (like keyboard, gamepad, etc.), you assign them to your gameplay actions,
like "Move" or "Jump" which are just regular structs. Actions are assigned to contexts, which are components that represent the current
state of the player character, like "OnFoot" or "InCar".

For example, if you have a player character that can be on foot or drive a car, you can swap the context to have the same keys
perform different actions. On foot, pressing <kbd>Space</kbd> could make the character jump, while when driving, pressing <kbd>Space</kbd>
should act as a brake.

Entities can have any number of contexts, with evaluation order controlled by a defined priority. Actions can also consume inputs,
allowing you to layer behaviors on top of each other.

Instead of reacting to raw input data like "Released" or "Pressed", the crate provides modifiers and conditions.

Modifiers let you change the input before passing it to the action. We provide common modifiers, like dead zones, inversion, and scaling,
but you can add your own by implementing a trait.

Conditions define how an action activates. We also provide built-in conditions, such as "Pressed", "Released", "Hold for n secs", etc.
However, you can also add gameplay-based conditions like "Can jump" by implementing a trait.

To respond to action changes, you can use observers. For example, with the condition "Hold for 1 sec", you can trigger a strong attack
on "Completed", or a regular attack on "Cancelled".

## Bevy compatibility

| bevy        | bevy_enhanced_input |
| ----------- | ------------------- |
| 0.14.0      | unreleased          |
