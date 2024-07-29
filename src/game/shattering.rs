use std::time::Duration;

use bevy::{
    app::{App, Update},
    asset::{Assets, Handle},
    core::Name,
    math::Vec3,
    pbr::{PbrBundle, StandardMaterial},
    prelude::{
        Commands, Component, DespawnRecursiveExt, Entity, Event, IntoSystemConfigs, Mesh, Query,
        Res, ResMut, StateScoped, Transform, Trigger,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
    utils::default,
};
use bevy_ghx_destruction::slicing::slicing::slice_bevy_mesh_iterative;
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, ComputedColliderShape, ExternalImpulse,
    Friction, Restitution, RigidBody,
};

use crate::{screen::Screen, AppSet};

pub const SHARDS_DESPAWN_DELAY_MS: u64 = 3000;
pub const SHATTER_ITERATION_COUNT: u32 = 6;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, ((despawn_shards).in_set(AppSet::Update),));

    app.observe(shatter_entity);
}

#[derive(Event, Debug, Clone, Reflect)]
pub struct ShatterEntity {
    pub entity: Entity,
    pub impulse: Vec3,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
struct Shard {
    despawn_timer: Timer,
}

// TODO Spawn a parent entity for shattered pieces : add the timer to the parent only
fn shatter_entity(
    trigger: Trigger<ShatterEntity>,
    mut commands: Commands,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    mut meshes_assets: ResMut<Assets<Mesh>>,
    shattered_entity_query: Query<(&Transform, &Handle<StandardMaterial>, &Handle<Mesh>)>,
) {
    let shatter_info = trigger.event();

    // Despawn the entity
    commands.entity(shatter_info.entity).despawn();

    let Ok((transform, mat_handle, mesh_handle)) = shattered_entity_query.get(shatter_info.entity)
    else {
        return;
    };
    let Some(mesh_to_shatter) = meshes_assets.get(mesh_handle) else {
        return;
    };

    // // TODO Link to parent entity with a despawn timer
    // let shards_parent = commands
    //     .spawn(Shard {
    //         despawn_timer: Timer::new(
    //             Duration::from_millis(SHARDS_DESPAWN_DELAY_MS),
    //             TimerMode::Once,
    //         ),
    //     })
    //     .id();

    let shards = slice_bevy_mesh_iterative(mesh_to_shatter, SHATTER_ITERATION_COUNT, None);
    for shard_mesh in shards {
        let Some(collider) =
            Collider::from_bevy_mesh(&shard_mesh, &ComputedColliderShape::ConvexHull)
        else {
            continue;
        };
        // let Some(aabb) = shard_mesh.compute_aabb() else {
        //     continue;
        // };
        let mesh_handle = meshes_assets.add(shard_mesh.clone());
        //   let shard_entity =
        let shard_entity = commands
            .spawn((
                Name::new("Shard"),
                StateScoped(Screen::Playing),
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    transform: Transform::from(*transform),
                    material: mat_handle.clone(),
                    ..default()
                },
                // Physics
                RigidBody::Dynamic,
                collider,
                ActiveCollisionTypes::default(),
                Friction::coefficient(0.7),
                Restitution::coefficient(0.05),
                ColliderMassProperties::Density(2.0),
                // Logic
                Shard {
                    despawn_timer: Timer::new(
                        Duration::from_millis(SHARDS_DESPAWN_DELAY_MS),
                        TimerMode::Once,
                    ),
                },
            ))
            .id();

        // commands.entity(shards_parent).add_child(shard_entity);

        // TODO May add impulse to shards
        // let frag_center: Vec3 = aabb.center.into();
        // let separating_impulse =
        //     FRAGMENTS_SEPARATION_IMPULSE_FACTOR * (frag_center - shatter_info.slice_positions.0);

        commands.entity(shard_entity).insert(ExternalImpulse {
            impulse: shatter_info.impulse,
            torque_impulse: Vec3::ZERO,
        });
    }
}

fn despawn_shards(
    mut commands: Commands,
    time: Res<Time>,
    mut fragments_query: Query<(Entity, &mut Shard)>,
) {
    for (entity, mut shards) in fragments_query.iter_mut() {
        shards.despawn_timer.tick(time.delta());
        if shards.despawn_timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
