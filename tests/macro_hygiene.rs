#![expect(
    dead_code,
    reason = "if any of these types are used, \
    then the macro is using one of these types, \
    instead of the actual `bevy_enhanced_input` type, \
    which breaks macro hygiene"
)]

struct InputAction;
struct Accumulation;
struct ActionValueDim;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Bool, accumulation = Cumulative)]
struct Dummy1;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Axis1D)]
struct Dummy2;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Axis2D)]
struct Dummy3;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Axis3D)]
struct Dummy4;
