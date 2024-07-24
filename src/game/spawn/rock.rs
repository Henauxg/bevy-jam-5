use bevy::{gltf::GltfMesh, prelude::*};

use crate::{
    game::assets::{GltfKey, HandleMap},
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_rock);
}

#[derive(Event, Debug)]
pub struct SpawnRock;

#[derive(Component)]
struct Sliceable;

fn spawn_rock(
    _trigger: Trigger<SpawnRock>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltf_handles: Res<HandleMap<GltfKey>>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
) {
    let gltf_handle = gltf_handles[&GltfKey::Rock].clone_weak();
    let Some(gltf) = assets_gltf.get(&gltf_handle) else {
        return;
    };
    let gltf_mesh = assets_gltfmesh.get(&gltf.meshes[0]).unwrap();
    commands.spawn((
        Name::new("Rock"),
        StateScoped(Screen::Mine),
        PbrBundle {
            mesh: gltf_mesh.primitives[0].mesh.clone(),
            // material: materials.add(Color::BLACK), // DARK_GREY.into()
            // LinearRgba::new(0.663, 0.663, 0.663, 1.).into()
            material: materials.add(Color::srgb_u8(50, 50, 50)),
            transform: Transform::from_xyz(0.0, 0., 0.0),
            ..default()
        },
        Sliceable,
    ));
}
