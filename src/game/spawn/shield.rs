use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, Friction, Restitution,
};

use crate::game::{
    arena::ArenaMode,
    assets::{HandleMap, SceneKey},
    player_animation::{EquipmentSlot, EquipmentToAttach},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_shield);
    app.register_type::<Shield>();
}

#[derive(Event, Debug)]
pub struct SpawnShield;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct Shield;

#[derive(Resource)]
pub struct ShieldCachedData {
    pub collider: Collider,
}

fn spawn_shield(
    _trigger: Trigger<SpawnShield>,
    mut commands: Commands,
    scenes_handles: Res<HandleMap<SceneKey>>,
    cached_data: Res<ShieldCachedData>,
) {
    commands.spawn((
        Name::new("Shield"),
        StateScoped(ArenaMode::Shield),
        SceneBundle {
            scene: scenes_handles[&SceneKey::Shield].clone_weak(),
            transform: Transform::IDENTITY,
            ..default()
        },
        // Physic
        // RigidBody::KinematicPositionBased,
        cached_data.collider.clone(),
        ActiveCollisionTypes::default(),
        Friction::coefficient(0.7),
        Restitution::coefficient(0.05),
        ColliderMassProperties::Density(2.0),
        // Logic
        Shield,
        EquipmentToAttach {
            slot: EquipmentSlot::Shield,
        },
    ));
}
