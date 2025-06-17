use core::time::Duration;

#[cfg(test)]
use bevy::ecs::system::SystemState;
use bevy::{ecs::system::SystemParam, prelude::*};

/// Time resources used for input conditions and modifier evaluation.
///
/// Dereferences to [`Self::current`], which is the default time resource
/// based on the current schedule. But you can optionally use [`Self::real`]
/// if you want the time to be unaffected by time dilation.
#[derive(SystemParam, Deref)]
pub struct InputTime<'w> {
    #[deref]
    pub current: Res<'w, Time>,
    pub real: Res<'w, Time<Real>>,
}

impl InputTime<'_> {
    /// Returns the delta of the time resource corresponding to the given [`TimeKind`].
    #[must_use]
    pub fn delta_kind(&self, kind: TimeKind) -> Duration {
        match kind {
            TimeKind::Virtual => self.current.delta(),
            TimeKind::Real => self.real.delta(),
        }
    }
}

/// Type of time resource to use.
#[derive(Debug, Clone, Copy, Default)]
pub enum TimeKind {
    /// Corresponds to [`Time`].
    ///
    /// Virtual game time, affected by [`Time::pause`] and [`Time::relative_speed`].
    #[default]
    Virtual,
    /// Corresponds to [`Time<Real>`].
    ///
    /// Real wall-clock time elapsed, not affected by pausing or scaling.
    Real,
}

/// Helper for tests to simplify [`InputTime`] creation.
#[cfg(test)]
pub(crate) fn init_world<'w>() -> (World, SystemState<InputTime<'w>>) {
    let mut world = World::new();
    world.init_resource::<Time>();
    world.init_resource::<Time<Real>>();

    let state = SystemState::<InputTime>::new(&mut world);

    (world, state)
}
