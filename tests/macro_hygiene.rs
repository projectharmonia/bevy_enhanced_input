#![expect(
    dead_code,
    reason = "if any of these types are used, \
    then the macro is using one of these types, \
    instead of the actual `bevy_enhanced_input` type, \
    which breaks macro hygiene"
)]

struct Accumulation;
struct InputAction;
struct ActionValueDim;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Bool)]
struct Foo1;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Axis1D)]
struct Foo2;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Axis2D)]
struct Foo3;

#[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
#[input_action(dim = Axis3D)]
struct Foo4;
