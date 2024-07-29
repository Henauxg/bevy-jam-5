use bevy::prelude::*;

use crate::game::{
    arena::ArenaMode,
    assets::{HandleMap, SceneKey, ASSETS_SCALE, GLADIATOR_ASSETS_SCALE},
    shield::throwers::Thrower,
};

use super::jug::SpawnJug;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_jug_thrower);
}

#[derive(Event, Debug)]
pub struct SpawnJugThrower {
    pub pos: Vec3,
    pub looking_at: Vec3,
}

fn spawn_jug_thrower(
    trigger: Trigger<SpawnJugThrower>,
    mut commands: Commands,
    scenes_handles: Res<HandleMap<SceneKey>>,
) {
    let spawn_info = trigger.event();
    commands
        .spawn((
            Name::new("Jug Thrower"),
            StateScoped(ArenaMode::Shield),
            SceneBundle {
                scene: scenes_handles[&SceneKey::Gladiator].clone_weak(),
                transform: Transform::from_translation(spawn_info.pos)
                    .looking_at(spawn_info.looking_at, Vec3::Y)
                    .with_scale(Vec3::splat(GLADIATOR_ASSETS_SCALE)),
                ..default()
            },
            Thrower,
        ))
        // Could share the observer but we really don't have many jug throwers
        .observe(throw_jug);
}

#[derive(Event, Debug)]
pub struct ThrowJug {
    pub at: Vec3,
}

fn throw_jug(
    trigger: Trigger<ThrowJug>,
    mut commands: Commands,
    transforms_query: Query<&Transform, With<Thrower>>,
) {
    let throw_info = trigger.event();

    let Ok(thrower_transform) = transforms_query.get(trigger.entity()) else {
        return;
    };
    commands.trigger(SpawnJug {
        pos: thrower_transform.translation,
        target: throw_info.at,
    });
}
