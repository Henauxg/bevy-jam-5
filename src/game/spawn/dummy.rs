use bevy::{gltf::GltfMesh, prelude::*};
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, Friction, Restitution, RigidBody,
};

use crate::{
    game::{
        assets::{GltfKey, HandleMap, ASSETS_SCALE},
        dummies::slicing::Sliceable,
    },
    screen::Screen,
};

use super::arena::DEFAULT_GLADIATOR_POS;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_dummy);
    app.register_type::<Dummy>();
}

#[derive(Event, Debug)]
pub struct SpawnDummy {
    pub pos: Vec3,
    pub slot_index: usize,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Dummy(pub usize);

#[derive(Resource)]
pub struct DummyCachedData {
    pub collider: Collider,
}

fn spawn_dummy(
    trigger: Trigger<SpawnDummy>,
    mut commands: Commands,
    dummy_cached_data: Res<DummyCachedData>,
    gltf_handles: Res<HandleMap<GltfKey>>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // TODO Cache this in a Resource
    let gltf_handle = &gltf_handles[&GltfKey::Dummy];
    let Some(gltf) = assets_gltf.get(gltf_handle) else {
        return;
    };
    let Some(gltf_mesh) = assets_gltfmesh.get(&gltf.meshes[0]) else {
        return;
    };
    let mesh_handle = &gltf_mesh.primitives[0].mesh;
    let mat_handle = &gltf.materials[0];

    let spawn_info = trigger.event();
    commands.spawn((
        Name::new("Dummy"),
        StateScoped(Screen::Playing),
        PbrBundle {
            // scene: scenes_handles[&SceneKey::Gladiator].clone_weak(),
            mesh: mesh_handle.clone(),
            // TODO Material
            // material: materials.add(Color::srgb_u8(50, 50, 50)),
            material: mat_handle.clone(),
            transform: Transform::from_translation(spawn_info.pos)
                .looking_at(DEFAULT_GLADIATOR_POS, Vec3::Y)
                .with_scale(Vec3::splat(ASSETS_SCALE)),
            ..default()
        },
        // Physic
        RigidBody::Fixed,
        dummy_cached_data.collider.clone(),
        ActiveCollisionTypes::default(),
        Friction::coefficient(0.7),
        Restitution::coefficient(0.05),
        ColliderMassProperties::Density(2.0),
        // Logic
        Sliceable,
        Dummy(spawn_info.slot_index),
    ));
    // }
}
