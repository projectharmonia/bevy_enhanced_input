#![expect(
    dead_code,
    reason = "if any of these types are used, \
    then the macro is using one of these types, \
    instead of the actual `bevy_enhanced_input` type, \
    which breaks macro hygiene"
)]

use bevy::prelude::{Vec2, Vec3};

struct InputAction;
struct Accumulation;
struct ActionValueDim;
struct InputContext;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = bool, accumulation = Cumulative)]
struct Action1;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = f32)]
struct Action2;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = Vec2)]
struct Action3;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = Vec3)]
struct Action4;

#[derive(Debug, bevy_enhanced_input::prelude::InputContext)]
struct Marker;
