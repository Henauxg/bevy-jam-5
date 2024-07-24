//! Spawn the player.

use bevy::{gltf::GltfMesh, prelude::*};

use crate::{
    game::{
        assets::{GltfKey, HandleMap, SceneKey},
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

fn spawn_dummy(
    trigger: Trigger<SpawnDummy>,
    mut commands: Commands,
    // gltf_handles: Res<HandleMap<GltfKey>>,
    // scenes_handles: Res<HandleMap<SceneKey>>,
    gltf_handles: Res<HandleMap<GltfKey>>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
) {
    let gltf_handle = gltf_handles[&GltfKey::Dummy].clone_weak();
    let Some(gltf) = assets_gltf.get(&gltf_handle) else {
        return;
    };
    let gltf_mesh = assets_gltfmesh.get(&gltf.meshes[0]).unwrap();

    // TODO Check slots
    let spawn_info = trigger.event();
    for i in 0..spawn_info.count {
        commands.spawn((
            Name::new("Dummy"),
            StateScoped(Screen::Playing),
            PbrBundle {
                // scene: scenes_handles[&SceneKey::Gladiator].clone_weak(),
                mesh: gltf_mesh.primitives[0].mesh.clone(),
                // TODO Material
                transform: Transform::from_translation(DUMMY_POSITIONS[i])
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            Sliceable,
        ));
    }
}
