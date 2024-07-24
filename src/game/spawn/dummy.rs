use bevy::{gltf::GltfMesh, prelude::*};
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, Friction, Restitution, RigidBody,
};

use crate::{
    game::{
        assets::{GltfKey, HandleMap},
        slicing::Sliceable,
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_dummy);
    app.register_type::<Dummy>();
}

#[derive(Event, Debug)]
pub struct SpawnDummy {
    pub count: usize,
}

pub const DUMMY_POSITIONS: [Vec3; 3] = [
    Vec3::new(0., 0., 1.5),
    Vec3::new(-1.5, 0., 1.25),
    Vec3::new(-3., 0., 1.),
];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Dummy;

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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gltf_handle = &gltf_handles[&GltfKey::Dummy];
    let Some(gltf) = assets_gltf.get(gltf_handle) else {
        return;
    };
    let Some(gltf_mesh) = assets_gltfmesh.get(&gltf.meshes[0]) else {
        return;
    };
    let mesh_handle = &gltf_mesh.primitives[0].mesh;

    // TODO Check slots
    let spawn_info = trigger.event();
    for i in 0..spawn_info.count {
        commands.spawn((
            Name::new("Dummy"),
            StateScoped(Screen::Playing),
            PbrBundle {
                // scene: scenes_handles[&SceneKey::Gladiator].clone_weak(),
                mesh: mesh_handle.clone(),
                // TODO Material
                material: materials.add(Color::srgb_u8(50, 50, 50)),
                transform: Transform::from_translation(DUMMY_POSITIONS[i])
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            Sliceable,
            RigidBody::Fixed,
            dummy_cached_data.collider.clone(),
            ActiveCollisionTypes::default(),
            Friction::coefficient(0.7),
            Restitution::coefficient(0.05),
            ColliderMassProperties::Density(2.0),
        ));
    }
}
