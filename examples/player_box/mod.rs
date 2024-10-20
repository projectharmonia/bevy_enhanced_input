//! Simple character controller made with gizmos.

use bevy::prelude::*;

pub(super) struct PlayerBoxPlugin;

impl Plugin for PlayerBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, Self::update_position);
    }
}

impl PlayerBoxPlugin {
    fn update_position(
        mut gizmos: Gizmos,
        players: Query<(&Visibility, &Transform, &PlayerColor)>,
    ) {
        for (visibility, transform, color) in &players {
            if visibility != Visibility::Hidden {
                gizmos.rect(
                    transform.translation,
                    transform.rotation,
                    transform.scale.xy(),
                    color.0,
                );
            }
        }
    }
}

pub(super) const DEFAULT_SPEED: f32 = 400.0;

#[derive(Bundle)]
pub(super) struct PlayerBoxBundle {
    pub(super) color: PlayerColor,
    pub(super) visibility: Visibility,
    pub(super) player: PlayerBox,
    pub(super) transform: Transform,
}

impl Default for PlayerBoxBundle {
    fn default() -> Self {
        Self {
            color: Default::default(),
            visibility: Default::default(),
            player: Default::default(),
            transform: Transform::from_scale(Vec3::splat(50.0)),
        }
    }
}

#[derive(Component, Default)]
pub(super) struct PlayerBox;

#[derive(Component, Default)]
pub(super) struct PlayerColor(pub(super) Color);
