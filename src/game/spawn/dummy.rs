use std::time::Duration;

use bevy::{gltf::GltfMesh, prelude::*};
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, Friction, Restitution, RigidBody,
};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::{
    game::{
        arena::ArenaMode,
        assets::{GltfKey, HandleMap, ASSETS_SCALE},
        sword::slicing::Sliceable,
    },
    screen::Screen,
};

use super::arena::DEFAULT_GLADIATOR_POS;

// TODO Consider decreasing with difficulty
pub const DEFAULT_DUMMY_DESPAWN_TIMER_MS: u64 = 3000;

pub const DUMMY_FALL_ANIMATION_DURATION_MS: u64 = 1000;
pub const DUMMY_FALL_START_UP_DELTA: f32 = 25.;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_dummy);
    app.register_type::<Dummy>();

    app.add_systems(Update, attach_dummy_logic.run_if(in_state(Screen::Playing)));
}

#[derive(Event, Debug)]
pub struct SpawnDummy {
    pub pos: Vec3,
    pub slot_index: usize,
    pub scope: ArenaMode,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Dummy {
    pub slot_index: usize,
    pub despawn_timer: Timer,
}

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

    let fall_animation = Tween::new(
        EaseFunction::ExponentialIn,
        Duration::from_millis(DUMMY_FALL_ANIMATION_DURATION_MS),
        TransformPositionLens {
            start: spawn_info.pos + DUMMY_FALL_START_UP_DELTA * Vec3::Y,
            end: spawn_info.pos,
        },
    );

    commands.spawn((
        Name::new("Dummy"),
        StateScoped(spawn_info.scope),
        PbrBundle {
            mesh: mesh_handle.clone(),
            // TODO Material
            // material: materials.add(Color::srgb_u8(50, 50, 50)),
            material: mat_handle.clone(),
            transform: Transform::from_translation(
                spawn_info.pos + DUMMY_FALL_START_UP_DELTA * Vec3::Y,
            )
            .looking_at(
                DEFAULT_GLADIATOR_POS + DUMMY_FALL_START_UP_DELTA * Vec3::Y,
                Vec3::Y,
            )
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
        AttachDummyLogic {
            timer: Timer::new(
                Duration::from_millis(DUMMY_FALL_ANIMATION_DURATION_MS),
                TimerMode::Once,
            ),
            slot_index: spawn_info.slot_index,
        },
        // Animation
        Animator::new(fall_animation),
    ));
}

#[derive(Component, Debug, Reflect)]
struct AttachDummyLogic {
    timer: Timer,
    slot_index: usize,
}

fn attach_dummy_logic(
    mut commands: Commands,
    time: Res<Time>,
    mut spawning_dummies_query: Query<(Entity, &mut AttachDummyLogic)>,
) {
    for (entity, mut spawning_dummy) in spawning_dummies_query.iter_mut() {
        spawning_dummy.timer.tick(time.delta());
        if spawning_dummy.timer.finished() {
            commands
                .entity(entity)
                .remove::<AttachDummyLogic>()
                .insert((
                    Sliceable,
                    Dummy {
                        slot_index: spawning_dummy.slot_index,
                        despawn_timer: Timer::new(
                            Duration::from_millis(DEFAULT_DUMMY_DESPAWN_TIMER_MS),
                            TimerMode::Once,
                        ),
                    },
                ));
        }
    }
}
