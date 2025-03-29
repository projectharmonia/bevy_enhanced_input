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
struct ActionsMarker;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = bool, accumulation = Cumulative)]
struct Dummy1;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = f32)]
struct Dummy2;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = Vec2)]
struct Dummy3;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(output = Vec3)]
struct Dummy4;

#[derive(Debug, bevy_enhanced_input::prelude::ActionsMarker)]
struct Marker;
