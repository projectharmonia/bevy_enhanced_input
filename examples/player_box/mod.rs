//! Simple character controller made with gizmos.

use bevy::prelude::*;

pub(crate) struct PlayerBoxPlugin;

impl Plugin for PlayerBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_position);
    }
}

fn update_position(mut gizmos: Gizmos, players: Query<(&Visibility, &Transform, &PlayerColor)>) {
    for (visibility, transform, color) in &players {
        if visibility != Visibility::Hidden {
            const DEFAULT_SCALE: Vec2 = Vec2::splat(50.0);
            gizmos.rect(
                Isometry3d::new(transform.translation, transform.rotation),
                DEFAULT_SCALE + transform.scale.xy(),
                color.0,
            );
        }
    }
}

pub(crate) const DEFAULT_SPEED: f32 = 10.0;

#[derive(Component, Default)]
#[require(PlayerColor, Visibility, Transform)]
pub(crate) struct PlayerBox;

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct PlayerColor(pub(crate) Color);
