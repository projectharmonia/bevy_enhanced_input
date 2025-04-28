use bevy::prelude::*;

use crate::{
    input_binding::{BindingBuilder, InputBinding, IntoBindings},
    input_modifier::{negate::Negate, swizzle_axis::SwizzleAxis},
};

/// A preset to map buttons as 2-dimensional input.
///
/// Uses [`SwizzleAxis`] and [`Negate`] to bind inputs to cardinal directions.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space for [`Transform::translation`],
/// you will need to invert Y and apply it to Z inside your observer.
///
/// See also [`Axial`], [`Bidirectional`] and [`Spatial`].
///
/// # Examples
///
/// Map keyboard inputs into a 2D movement action.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn binding(
///     trigger: Trigger<Binding<Player>>,
///     settings: Res<KeyboardSettings>,
///     mut players: Query<&mut Actions<Player>>,
/// ) {
///     let mut actions = players.get_mut(trigger.target()).unwrap();
///     actions.bind::<Move>().to(Cardinal {
///         north: &settings.forward,
///         east: &settings.right,
///         south: &settings.backward,
///         west: &settings.left,
///     });
/// }
///
/// // We use `KeyCode` here because we are only interested in key presses.
/// // But you can also use `Input` if you want to e.g.
/// // combine mouse and keyboard input sources.
/// #[derive(Resource)]
/// struct KeyboardSettings {
///     forward: Vec<KeyCode>,
///     right: Vec<KeyCode>,
///     backward: Vec<KeyCode>,
///     left: Vec<KeyCode>,
/// }
///
/// #[derive(InputContext)]
/// struct Player;
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec2)]
/// struct Move;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Cardinal<I: IntoBindings> {
    pub north: I,
    pub east: I,
    pub south: I,
    pub west: I,
}

impl Cardinal<KeyCode> {
    /// Maps WASD keys as 2-dimensional input.
    ///
    /// See also [`Self::arrow_keys`].
    #[must_use]
    pub fn wasd_keys() -> Self {
        Self {
            north: KeyCode::KeyW,
            west: KeyCode::KeyA,
            south: KeyCode::KeyS,
            east: KeyCode::KeyD,
        }
    }

    /// Maps keyboard arrow keys as 2-dimensional input.
    ///
    /// See also [`Self::wasd_keys`].
    #[must_use]
    pub fn arrow_keys() -> Self {
        Self {
            north: KeyCode::ArrowUp,
            west: KeyCode::ArrowLeft,
            south: KeyCode::ArrowDown,
            east: KeyCode::ArrowRight,
        }
    }
}

impl Cardinal<GamepadButton> {
    /// Maps D-pad as 2-dimensional input.
    ///
    /// See also [`Self::wasd_keys`].
    #[must_use]
    pub fn dpad_buttons() -> Self {
        Self {
            north: GamepadButton::DPadUp,
            west: GamepadButton::DPadLeft,
            south: GamepadButton::DPadDown,
            east: GamepadButton::DPadRight,
        }
    }
}

impl<I: IntoBindings> IntoBindings for Cardinal<I> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        // Y
        let north = self
            .north
            .into_bindings()
            .map(|binding| binding.with_modifiers(SwizzleAxis::YXZ));

        // -X
        let west = self
            .west
            .into_bindings()
            .map(|binding| binding.with_modifiers(Negate::all()));

        // -Y
        let south = self
            .south
            .into_bindings()
            .map(|binding| binding.with_modifiers((Negate::all(), SwizzleAxis::YXZ)));

        // X
        let east = self.east.into_bindings();

        north.chain(east).chain(south).chain(west)
    }
}

/// A preset to map axes as 2-dimensional input.
///
/// Uses [`SwizzleAxis`] to bind inputs to axes.
///
/// See also [`Cardinal`].
///
/// # Examples
///
/// Maps gamepad axes into a 2D movement action.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn binding(
///     trigger: Trigger<Binding<Player>>,
///     settings: Res<GamepadSettings>,
///     mut players: Query<&mut Actions<Player>>,
/// ) {
///     let mut actions = players.get_mut(trigger.target()).unwrap();
///     actions.bind::<Move>().to(Axial {
///         x: &settings.horizontal_movement,
///         y: &settings.vertical_movement,
///     });
/// }
///
/// #[derive(Resource)]
/// struct GamepadSettings {
///     horizontal_movement: Vec<GamepadAxis>,
///     vertical_movement: Vec<GamepadAxis>,
/// }
///
/// #[derive(InputContext)]
/// struct Player;
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec2)]
/// struct Move;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Axial<I: IntoBindings> {
    pub x: I,
    pub y: I,
}

impl Axial<GamepadAxis> {
    /// Maps left stick as 2-dimensional input.
    ///
    /// See also [`Self::right_stick`].
    pub fn left_stick() -> Self {
        Self {
            x: GamepadAxis::LeftStickX,
            y: GamepadAxis::LeftStickY,
        }
    }

    /// Maps right stick as 2-dimensional input.
    ///
    /// See also [`Self::left_stick`].
    pub fn right_stick() -> Self {
        Self {
            x: GamepadAxis::RightStickX,
            y: GamepadAxis::RightStickY,
        }
    }
}

impl<I: IntoBindings> IntoBindings for Axial<I> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        let x = self.x.into_bindings();
        let y = self
            .y
            .into_bindings()
            .map(|binding| binding.with_modifiers(SwizzleAxis::YXZ));

        x.chain(y)
    }
}

/// A preset to map buttons as 1-dimensional input.
///
/// Positive binding will be passed as is and negative will be reversed using [`Negate`].
///
/// See also [`Cardinal`] and [`Spatial`].
#[derive(Debug, Clone, Copy)]
pub struct Bidirectional<I: IntoBindings> {
    pub positive: I,
    pub negative: I,
}

impl<I: IntoBindings> IntoBindings for Bidirectional<I> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        let positive = self.positive.into_bindings();
        let negative = self
            .negative
            .into_bindings()
            .map(|binding| binding.with_modifiers(Negate::all()));

        positive.chain(negative)
    }
}

/// A preset to map buttons as 3-dimensional input.
///
/// Uses [`SwizzleAxis`] and [`Negate`] to bind inputs to the Y and Z directions.
///
/// See also [`Cardinal`] and [`Bidirectional`].
///
/// # Examples
///
/// Map keyboard inputs into a 3D movement action.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// fn binding(
///     trigger: Trigger<Binding<FlyCamera>>,
///     settings: Res<KeyboardSettings>,
///     mut cameras: Query<&mut Actions<FlyCamera>>,
/// ) {
///     let mut actions = cameras.get_mut(trigger.target()).unwrap();
///     actions.bind::<Move>().to(Spatial {
///         forward: &settings.forward,
///         right: &settings.right,
///         backward: &settings.backward,
///         left: &settings.left,
///         up: &settings.up,
///         down: &settings.down,
///     });
/// }
///
/// // We use `KeyCode` here because we are only interested in key presses.
/// // But you can also use `Input` if you want to e.g.
/// // combine mouse and keyboard input sources.
/// #[derive(Resource)]
/// struct KeyboardSettings {
///     forward: Vec<KeyCode>,
///     right: Vec<KeyCode>,
///     backward: Vec<KeyCode>,
///     left: Vec<KeyCode>,
///     up: Vec<KeyCode>,
///     down: Vec<KeyCode>,
/// }
///
/// #[derive(InputContext)]
/// struct FlyCamera;
///
/// #[derive(Debug, InputAction)]
/// #[input_action(output = Vec3)]
/// struct Move;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Spatial<I: IntoBindings> {
    pub forward: I,
    pub backward: I,
    pub left: I,
    pub right: I,
    pub up: I,
    pub down: I,
}

impl Spatial<KeyCode> {
    /// Maps WASD keys for horizontal (XZ) inputs and takes in up/down mappings.
    ///
    /// See also [`Self::arrows_and`].
    pub fn wasd_and(up: KeyCode, down: KeyCode) -> Self {
        Spatial {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            up,
            down,
        }
    }

    /// Maps arrow keys for horizontal (XZ) inputs and takes in up/down mappings.
    ///
    /// See also [`Self::wasd_and`].
    pub fn arrows_and(up: KeyCode, down: KeyCode) -> Self {
        Spatial {
            forward: KeyCode::ArrowUp,
            backward: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
            up,
            down,
        }
    }
}

impl<I: IntoBindings> IntoBindings for Spatial<I> {
    fn into_bindings(self) -> impl Iterator<Item = InputBinding> {
        // Z
        let backward = self
            .backward
            .into_bindings()
            .map(|binding| binding.with_modifiers(SwizzleAxis::ZYX));

        // -Z
        let forward = self
            .forward
            .into_bindings()
            .map(|binding| binding.with_modifiers((Negate::all(), SwizzleAxis::ZYX)));

        // X
        let right = self.right.into_bindings();

        // -X
        let left = self
            .left
            .into_bindings()
            .map(|binding| binding.with_modifiers(Negate::all()));

        // Y
        let up = self
            .up
            .into_bindings()
            .map(|binding| binding.with_modifiers(SwizzleAxis::YXZ));

        // -Y
        let down = self
            .down
            .into_bindings()
            .map(|binding| binding.with_modifiers((Negate::all(), SwizzleAxis::YXZ)));

        backward
            .chain(forward)
            .chain(right)
            .chain(left)
            .chain(up)
            .chain(down)
    }
}
