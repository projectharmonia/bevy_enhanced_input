pub use interpolation::EaseFunction;

use std::{any, fmt::Debug};

use bevy::prelude::*;
use interpolation::Ease;

use crate::action_value::{ActionValue, ActionValueDim};

/// Pre-processors that alter the raw input values.
///
/// Input modifiers are useful for applying sensitivity settings, smoothing input over multiple frames,
/// or changing how input behaves based on the state of the player.
///
/// Because you have access to the world when making your own modifier, you can access any game state you want.
///
/// Modifiers can be applied both to inputs and actions.
/// See [`ActionMap::with_modifier`](super::context_instance::ActionMap::with_modifier)
/// and [`InputMap::with_modifier`](super::context_instance::InputMap::with_modifier).
pub trait InputModifier: Sync + Send + Debug + 'static {
    /// Returns pre-processed value.
    ///
    /// Called each frame.
    fn apply(&mut self, world: &World, delta: f32, value: ActionValue) -> ActionValue;
}

macro_rules! ignore_incompatible {
    ($value:expr) => {
        warn_once!(
            "trying to apply `{}` to a `{:?}` value, which is not possible",
            any::type_name::<Self>(),
            $value.dim(),
        );
        return $value
    };
}

/// Input values within the range [Self::lower_threshold] -> [Self::upper_threshold] will be remapped from 0 -> 1.
/// Values outside this range will be clamped.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct DeadZone {
    pub kind: DeadZoneKind,

    /// Threshold below which input is ignored.
    pub lower_threshold: f32,

    /// Threshold below which input is ignored.
    pub upper_threshold: f32,
}

impl DeadZone {
    fn dead_zone(self, axis_value: f32) -> f32 {
        // Translate and scale the input to the +/- 1 range after removing the dead zone.
        let lower_bound = (axis_value.abs() - self.lower_threshold).max(0.0);
        let scaled_value = lower_bound / (self.upper_threshold - self.lower_threshold);
        scaled_value.min(1.0) * axis_value.signum()
    }
}

impl Default for DeadZone {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            lower_threshold: 0.2,
            upper_threshold: 1.0,
        }
    }
}

impl InputModifier for DeadZone {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => self.dead_zone(value).into(),
            ActionValue::Axis2D(mut value) => match self.kind {
                DeadZoneKind::Radial => {
                    (value.normalize_or_zero() * self.dead_zone(value.length())).into()
                }
                DeadZoneKind::Axial => {
                    value.x = self.dead_zone(value.x);
                    value.y = self.dead_zone(value.y);
                    value.into()
                }
            },
            ActionValue::Axis3D(mut value) => match self.kind {
                DeadZoneKind::Radial => {
                    (value.normalize_or_zero() * self.dead_zone(value.length())).into()
                }
                DeadZoneKind::Axial => {
                    value.x = self.dead_zone(value.x);
                    value.y = self.dead_zone(value.y);
                    value.z = self.dead_zone(value.z);
                    value.into()
                }
            },
        }
    }
}

/// Dead zone behavior.
#[derive(Default, Clone, Copy, Debug)]
pub enum DeadZoneKind {
    // Apply dead zone logic to all axes simultaneously.
    //
    // This gives smooth input (circular/spherical coverage). On a 1d axis input this works identically to [`Self::Axial`].
    #[default]
    Radial,

    // Apply dead zone to axes individually.
    //
    // This will result in input being chamfered at the corners for 2d/3d axis inputs.
    Axial,
}

/// Response curve exponential.
///
/// Apply a simple exponential response curve to input values, per axis.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct ExponentialCurve {
    /// Curve exponent.
    pub exponent: Vec3,
}

impl ExponentialCurve {
    fn curve(value: f32, exponent: f32) -> f32 {
        if value != 1.0 {
            value.signum() * value.abs().powf(exponent)
        } else {
            value
        }
    }
}

impl Default for ExponentialCurve {
    fn default() -> Self {
        Self {
            exponent: Vec3::ONE,
        }
    }
}

impl InputModifier for ExponentialCurve {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => Self::curve(value, self.exponent.x).into(),
            ActionValue::Axis2D(mut value) => {
                value.x = Self::curve(value.x, self.exponent.x);
                value.y = Self::curve(value.y, self.exponent.y);
                value.into()
            }
            ActionValue::Axis3D(mut value) => {
                value.x = Self::curve(value.x, self.exponent.x);
                value.y = Self::curve(value.y, self.exponent.y);
                value.y = Self::curve(value.z, self.exponent.z);
                value.into()
            }
        }
    }
}

/// Scales input by a set factor per axis.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct Scalar {
    /// The scalar that will be applied to the input value.
    ///
    /// For example, with the scalar set to `Vec3::new(2.0, 2.0, 2.0)`, each input axis will be multiplied by 2.0.
    ///
    /// Does nothing for boolean values.
    pub scalar: Vec3,
}

impl InputModifier for Scalar {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => (value * self.scalar.x).into(),
            ActionValue::Axis2D(value) => (value * self.scalar.xy()).into(),
            ActionValue::Axis3D(value) => (value * self.scalar).into(),
        }
    }
}

/// Multiplies the input value by delta time for this frame.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Clone, Copy, Debug)]
pub struct ScaleByDelta;

impl InputModifier for ScaleByDelta {
    fn apply(&mut self, _world: &World, delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) => {
                ignore_incompatible!(value);
            }
            ActionValue::Axis1D(value) => (value * delta).into(),
            ActionValue::Axis2D(value) => (value * delta).into(),
            ActionValue::Axis3D(value) => (value * delta).into(),
        }
    }
}

/// Smooth inputs out over multiple frames.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Debug)]
pub struct Smooth {
    /// How long input has been zero.
    zero_time: f32,

    /// Current average input/sample.
    average_value: Vec3,

    /// Number of samples since input has been zero.
    samples: u32,

    /// Input sampling total time.
    total_sample_time: f32,
}

impl InputModifier for Smooth {
    fn apply(&mut self, _world: &World, delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            ignore_incompatible!(value);
        }

        let mut sample_count: u8 = 1;
        if self.average_value.length_squared() != 0.0 {
            self.total_sample_time += delta;
            self.samples += sample_count as u32;
        }

        let mut value = value.as_axis3d();
        if delta < 0.25 {
            if self.samples > 0 && self.total_sample_time > 0.0 {
                // Seconds/sample.
                let axis_sampling_time = self.total_sample_time / self.samples as f32;
                debug_assert!(axis_sampling_time > 0.0);

                if value.length_squared() != 0.0 && sample_count > 0 {
                    self.zero_time = 0.0;
                    if self.average_value.length_squared() != 0.0 {
                        // This isn't the first tick with non-zero value.
                        if delta < axis_sampling_time * (sample_count as f32 + 1.0) {
                            // Smooth value so samples/tick is constant.
                            value *= delta / (axis_sampling_time * sample_count as f32);
                            sample_count = 1;
                        }
                    }

                    self.average_value = value * (1.0 / sample_count as f32);
                } else {
                    // No value received.
                    if self.zero_time < axis_sampling_time {
                        // Zero value is possibly because less than the value sampling interval has passed.
                        value = self.average_value * (delta / axis_sampling_time);
                    } else {
                        self.reset();
                    }

                    self.zero_time += delta; // increment length of time we've been at zero
                }
            }
        } else {
            // If we had an abnormally long frame, clear everything so it doesn't distort the results.
            self.reset();
        }

        ActionValue::Axis3D(value).convert(dim)
    }
}

impl Default for Smooth {
    fn default() -> Self {
        Self {
            zero_time: Default::default(),
            average_value: Default::default(),
            samples: Default::default(),
            total_sample_time: Self::DEFAULT_SAMPLE_TIME,
        }
    }
}

impl Smooth {
    const DEFAULT_SAMPLE_TIME: f32 = 0.0083;

    fn reset(&mut self) {
        self.zero_time = 0.0;
        self.average_value = Vec3::ZERO;
        self.samples = 0;
        self.total_sample_time = Self::DEFAULT_SAMPLE_TIME;
    }
}

/// Normalized smooth delta
///
/// Produces a smoothed normalized delta of the current(new) and last(old) input value.
///
/// Can't be applied to [`ActionValue::Bool`].
#[derive(Debug)]
pub struct SmoothDelta {
    /// Defines how value will be smoothed.
    pub smoothing_method: SmoothingMethod,

    /// Speed or alpha.
    ///
    /// If the speed given is 0, then jump to the target.
    pub speed: f32,

    old_value: Vec3,

    value_delta: Vec3,
}

impl SmoothDelta {
    #[must_use]
    pub fn new(smoothing_method: SmoothingMethod, speed: f32) -> Self {
        Self {
            smoothing_method,
            speed,
            old_value: Default::default(),
            value_delta: Default::default(),
        }
    }
}

impl InputModifier for SmoothDelta {
    fn apply(&mut self, _world: &World, delta: f32, value: ActionValue) -> ActionValue {
        let dim = value.dim();
        if dim == ActionValueDim::Bool {
            ignore_incompatible!(value);
        }

        let value = value.as_axis3d();
        let target_value_delta = (self.old_value - value).normalize_or_zero();
        self.old_value = value;

        let normalized_delta = delta / self.speed;
        self.value_delta = match self.smoothing_method {
            SmoothingMethod::EaseFunction(ease_function) => {
                let ease_delta = normalized_delta.calc(ease_function);
                self.value_delta.lerp(target_value_delta, ease_delta)
            }
            SmoothingMethod::Linear => self.value_delta.lerp(target_value_delta, normalized_delta),
        };

        ActionValue::Axis3D(self.value_delta).convert(dim)
    }
}

/// Behavior options for [`SmoothDelta`].
///
/// Describe how eased value should be computed.
#[derive(Clone, Copy, Debug)]
pub enum SmoothingMethod {
    /// Follow [`EaseFunction`].
    EaseFunction(EaseFunction),
    /// Linear interpolation, with no function.
    Linear,
}

/// Inverts value per axis.
///
/// By default, all axes are inverted.
#[derive(Debug)]
pub struct Negate {
    /// Wheter to inverse the X axis.
    pub x: bool,

    /// Wheter to inverse the Y axis.
    pub y: bool,

    /// Wheter to inverse the Z axis.
    pub z: bool,
}

impl Negate {
    /// Returns [`Self`] with invertion for all axes set to `invert`
    pub fn all(invert: bool) -> Self {
        Self {
            x: invert,
            y: invert,
            z: invert,
        }
    }

    /// Returns [`Self`] with invertion for x set to `invert`
    pub fn x(invert: bool) -> Self {
        Self {
            x: invert,
            y: false,
            z: false,
        }
    }

    /// Returns [`Self`] with invertion for y set to `invert`
    pub fn y(invert: bool) -> Self {
        Self {
            x: false,
            y: invert,
            z: false,
        }
    }

    /// Returns [`Self`] with invertion for z set to `invert`
    pub fn z(invert: bool) -> Self {
        Self {
            x: false,
            y: false,
            z: invert,
        }
    }
}

impl Default for Negate {
    fn default() -> Self {
        Self {
            x: true,
            y: true,
            z: true,
        }
    }
}

impl InputModifier for Negate {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        let x = if self.x { 1.0 } else { -1.0 };
        let y = if self.y { 1.0 } else { -1.0 };
        let z = if self.z { 1.0 } else { -1.0 };
        let negated = value.as_axis3d() * Vec3::new(x, y, z);

        ActionValue::Axis3D(negated).convert(value.dim())
    }
}

/// Swizzle axis components of an input value.
///
/// Useful to map a 1D input onto the Y axis of a 2D action.
///
/// Can't be applied to [`ActionValue::Bool`] and [`ActionValue::Axis1D`].
#[derive(Debug)]
pub enum SwizzleAxis {
    /// Swap X and Y axis. Useful for binding 1D inputs to the Y axis for 2D actions.
    YXZ,
    /// Swap X and Z axis.
    ZYX,
    /// Swap Y and Z axis.
    XZY,
    /// Reorder all axes, Y first.
    YZX,
    /// Reorder all axes, Z first.
    ZXY,
}

impl InputModifier for SwizzleAxis {
    fn apply(&mut self, _world: &World, _delta: f32, value: ActionValue) -> ActionValue {
        match value {
            ActionValue::Bool(_) | ActionValue::Axis1D(_) => {
                ignore_incompatible!(value);
            }
            ActionValue::Axis2D(value) => match self {
                SwizzleAxis::YXZ => value.yx().into(),
                SwizzleAxis::ZYX => Vec2::new(0.0, value.y).into(),
                SwizzleAxis::XZY => Vec2::new(value.x, 0.0).into(),
                SwizzleAxis::YZX => Vec2::new(value.y, 0.0).into(),
                SwizzleAxis::ZXY => Vec2::new(0.0, value.x).into(),
            },
            ActionValue::Axis3D(value) => match self {
                SwizzleAxis::YXZ => value.yxz().into(),
                SwizzleAxis::ZYX => value.zyx().into(),
                SwizzleAxis::XZY => value.xzy().into(),
                SwizzleAxis::YZX => value.yzx().into(),
                SwizzleAxis::ZXY => value.zxy().into(),
            },
        }
    }
}
