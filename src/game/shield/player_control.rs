use bevy::{
    app::{App, Update},
    math::Vec3,
    prelude::{in_state, IntoSystemConfigs, Query, Res, Transform, With},
    utils::default,
};
use bevy_mod_raycast::{cursor::CursorRay, prelude::Raycast};

use crate::{
    game::{arena::ArenaMode, spawn::player::Player},
    AppSet,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (rotate_player.in_set(AppSet::RecordInput)).run_if(in_state(ArenaMode::Shield)),
    );
}

fn rotate_player(
    cursor_ray: Res<CursorRay>,
    mut players_query: Query<&mut Transform, With<Player>>,
    mut raycast: Raycast,
) {
    let Ok(mut player_transform) = players_query.get_single_mut() else {
        return;
    };
    let Some(cursor_ray) = cursor_ray.0 else {
        return;
    };

    // TODO Raycast with mouse
    // raycast.debug_cast_ray(cursor_ray, &default(), &mut gizmos);
    let hits = raycast.cast_ray(cursor_ray, &default());
    let Some(hit) = hits.get(0) else {
        return;
    };

    let mut direction = hit.1.position() - player_transform.translation;
    // Quick & dirty, no rotation around x and z
    direction.y = 0.;
    player_transform.look_to(direction.normalize(), Vec3::Y);
}
