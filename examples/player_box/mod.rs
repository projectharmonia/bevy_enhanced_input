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
                const DEFAULT_SCALE: Vec2 = Vec2::splat(50.0);
                gizmos.rect(
                    transform.translation,
                    transform.rotation,
                    DEFAULT_SCALE + transform.scale.xy(),
                    color.0,
                );
            }
        }
    }
}

pub(super) const DEFAULT_SPEED: f32 = 10.0;

#[derive(Bundle, Default)]
pub(super) struct PlayerBoxBundle {
    pub(super) color: PlayerColor,
    pub(super) visibility: Visibility,
    pub(super) player: PlayerBox,
    pub(super) transform: Transform,
}

#[derive(Component, Default)]
pub(super) struct PlayerBox;

#[derive(Component, Default)]
pub(super) struct PlayerColor(pub(super) Color);
