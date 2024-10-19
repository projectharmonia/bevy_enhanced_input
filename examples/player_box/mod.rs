//! Simple character controller made with gizmos.

use bevy::prelude::*;

pub(super) struct PlayerBoxPlugin;

impl Plugin for PlayerBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, Self::update_position);
    }
}

impl PlayerBoxPlugin {
    fn update_position(mut gizmos: Gizmos, players: Query<(&Transform, &PlayerColor)>) {
        for (transform, color) in &players {
            gizmos.rect(
                transform.translation,
                transform.rotation,
                Vec2::ONE * 50.0,
                color.0,
            );
        }
    }
}

#[derive(Bundle, Default)]
pub(super) struct PlayerBoxBundle {
    pub(super) color: PlayerColor,
    pub(super) player: PlayerBox,
    pub(super) transform: Transform,
}

#[derive(Component, Default)]
pub(super) struct PlayerBox;

#[derive(Component, Default)]
pub(super) struct PlayerColor(Color);
