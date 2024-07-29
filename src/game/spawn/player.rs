use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, ColliderMassProperties, Friction, Restitution, RigidBody,
};

use crate::{
    game::assets::{HandleMap, SceneKey, ASSETS_SCALE},
    screen::Screen,
};

use super::helmet::SpawnHelmet;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_player);
    app.register_type::<Player>();
}

#[derive(Event, Debug)]
pub struct SpawnPlayer {
    pub pos: Vec3,
    pub looking_at: Vec3,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn spawn_player(
    trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    scenes_handles: Res<HandleMap<SceneKey>>,
) {
    let spawn_info = trigger.event();
    commands.spawn((
        Name::new("Gladiator"),
        StateScoped(Screen::Playing),
        SceneBundle {
            scene: scenes_handles[&SceneKey::Gladiator].clone_weak(),
            transform: Transform::from_translation(spawn_info.pos)
                .looking_at(spawn_info.looking_at, Vec3::Y)
                .with_scale(Vec3::splat(ASSETS_SCALE)),
            ..default()
        },
        // Physic
        // RigidBody::KinematicPositionBased,
        // cached_data.collider.clone(),
        ActiveCollisionTypes::default(),
        Friction::coefficient(0.7),
        Restitution::coefficient(0.05),
        ColliderMassProperties::Density(2.0),
        // Logic
        Player,
    ));

    commands.trigger(SpawnHelmet);
}
