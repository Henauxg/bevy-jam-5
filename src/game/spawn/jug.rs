use bevy::{gltf::GltfMesh, prelude::*};
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, ColliderMassProperties, ExternalImpulse,
    Friction, Restitution, RigidBody,
};

use crate::{
    game::assets::{GltfKey, HandleMap, ASSETS_SCALE},
    screen::Screen,
};

use super::arena::DEFAULT_GLADIATOR_POS;

pub const JUG_THROW_IMPULSE_FACTOR: f32 = 20.;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_jug);
    app.register_type::<Jug>();
}

#[derive(Event, Debug)]
pub struct SpawnJug {
    pub pos: Vec3,
    pub target: Vec3,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Jug;

#[derive(Resource)]
pub struct JugCachedData {
    pub collider: Collider,
}

fn spawn_jug(
    trigger: Trigger<SpawnJug>,
    mut commands: Commands,
    jug_cached_data: Res<JugCachedData>,
    gltf_handles: Res<HandleMap<GltfKey>>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
) {
    // TODO Cache this in a Resource
    let gltf_handle = &gltf_handles[&GltfKey::Jug1];
    let Some(gltf) = assets_gltf.get(gltf_handle) else {
        return;
    };
    let Some(gltf_mesh) = assets_gltfmesh.get(&gltf.meshes[0]) else {
        return;
    };
    let mesh_handle = &gltf_mesh.primitives[0].mesh;
    let mat_handle = &gltf.materials[0];

    let jug_throw = trigger.event();
    let jug_entity = commands
        .spawn((
            Name::new("Jug"),
            StateScoped(Screen::Playing),
            PbrBundle {
                mesh: mesh_handle.clone(),
                material: mat_handle.clone(),
                // TODO Gladiator height constant
                transform: Transform::from_translation(jug_throw.pos + 3. * Vec3::Y)
                    .looking_at(DEFAULT_GLADIATOR_POS, Vec3::Y)
                    .with_scale(Vec3::splat(ASSETS_SCALE)),
                ..default()
            },
            // Physic
            RigidBody::Dynamic,
            // TODO For now colliders are shared. Could have a simpler capsule collider or each have their own collider from mesh.
            jug_cached_data.collider.clone(),
            ActiveCollisionTypes::default(),
            Friction::coefficient(0.7),
            Restitution::coefficient(0.05),
            ColliderMassProperties::Density(2.0),
            ActiveEvents::COLLISION_EVENTS,
            // Logic
            Jug,
        ))
        // // TODO Could share the observer between all the jugs
        // .observe(shatter_jug)
        .id();

    let impulse = JUG_THROW_IMPULSE_FACTOR * (jug_throw.target - jug_throw.pos).normalize();
    // TODO Impulse/force/kinematic movmeent to target
    commands.entity(jug_entity).insert(ExternalImpulse {
        impulse,
        torque_impulse: Vec3::ZERO,
    });
}

// #[derive(Event, Debug)]
// pub struct ShatterJug;

// fn shatter_jug(shatter_event: Trigger<ShatterJug>) {

// }
