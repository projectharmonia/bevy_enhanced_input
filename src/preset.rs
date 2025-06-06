use bevy::prelude::*;

use crate::prelude::*;

/// A preset to 4 map buttons as 2-dimensional input.
///
/// In Bevy's 3D space, the -Z axis points forward and the +Z axis points
/// toward the camera. To map movement correctly in 3D space for [`Transform::translation`],
/// you will need to invert Y and apply it to Z inside your observer.
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
        let x = Bidirectional {
            positive: self.east,
            negative: self.west,
        };

        let y = Bidirectional {
            positive: self.north,
            negative: self.south,
        };

        x.into_bindings().chain(
            y.into_bindings()
                .map(|b| b.with_modifiers(SwizzleAxis::YXZ)),
        )
    }
}

/// A preset to map 2 axes as 2-dimensional input.
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
    pub fn left_stick() -> Self {
        Self {
            x: GamepadAxis::LeftStickX,
            y: GamepadAxis::LeftStickY,
        }
    }

    /// Maps right stick as 2-dimensional input.
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
            .map(|b| b.with_modifiers(SwizzleAxis::YXZ));

        x.chain(y)
    }
}

/// A preset to map 2 buttons as 1-dimensional input.
///
/// See [`Cardinal`] for a usage example.
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
            .map(|b| b.with_modifiers(Negate::all()));

        positive.chain(negative)
    }
}

/// A preset to map 6 buttons as 3-dimensional input.
///
/// See [`Cardinal`] for a usage example.
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
        let xy = Cardinal {
            north: self.up,
            east: self.right,
            south: self.down,
            west: self.left,
        };

        let z = Bidirectional {
            positive: self.backward,
            negative: self.forward,
        };

        xy.into_bindings().chain(
            z.into_bindings()
                .map(|b| b.with_modifiers(SwizzleAxis::ZYX)),
        )
    }
}
