use std::time::Duration;

use bevy::{
    app::{App, Update},
    asset::{Assets, Handle},
    core::Name,
    gltf::Gltf,
    input::ButtonInput,
    math::{Vec3, Vec3A},
    pbr::{PbrBundle, StandardMaterial},
    prelude::{
        default, Camera, Commands, Component, DespawnRecursiveExt, Entity, Event, Gizmos,
        GlobalTransform, IntoSystemConfigs, Mesh, MouseButton, Query, Res, ResMut, Resource,
        StateScoped, Transform, Trigger, With, Without,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
};
use bevy_ghx_destruction::{slicing::slicing::slice_bevy_mesh, types::Plane};
use bevy_mod_raycast::{cursor::CursorRay, prelude::Raycast};
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, ComputedColliderShape, ExternalImpulse,
    Friction, Restitution, RigidBody,
};

use crate::{
    game::assets::{GltfKey, HandleMap, ASSETS_SCALE},
    screen::Screen,
    AppSet,
};

pub const PLAYER_SLICE_FRAGMENTATION_DELAY_MS: u64 = 90;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Sliceable>();
    app.register_type::<SliceAttemptEvent>();
    app.register_type::<SliceEvent>();
    app.register_type::<SlicerState>();
    app.register_type::<FragmentationQueue>();

    app.add_systems(
        Update,
        (
            detect_slices.in_set(AppSet::RecordInput),
            (dequeue_fragmentations, despawn_fragments).in_set(AppSet::Update),
        ),
    );
    app.init_resource::<SlicerState>();
    app.init_resource::<FragmentationQueue>();

    app.observe(slice);
    app.observe(fragment_entity);
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
pub struct Sliceable;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
pub struct SlicedFragment {
    despawn_timer: Timer,
}
impl SlicedFragment {
    pub fn new() -> Self {
        Self {
            despawn_timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
        }
    }
}

#[derive(Event, Debug, Clone, Reflect)]
/// May not slice the entity, depending on the positions
struct SliceAttemptEvent {
    pub begin: Vec3,
    pub end: Vec3,
    pub entity: Entity,
}

#[derive(Event, Debug, Clone, Reflect)]
/// An entity is being sliced
pub struct SliceEvent {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone, Reflect)]
struct SpawnFragments {
    sliced_entity: Entity,
    sliced_object_transform: Transform,
    slice_positions: (Vec3, Vec3),
    fragments_meshes: [Mesh; 2],
}

#[derive(Resource, Default, Reflect)]
struct FragmentationQueue {
    queue: Vec<(SpawnFragments, Timer)>,
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
enum SlicerState {
    #[default]
    Idle,
    NoTarget,
    FirstHit(Vec3, Entity),
    Slicing {
        start: (Vec3, Entity),
        last_hit: Vec3,
    },
}

fn detect_slices(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    cursor_ray: Res<CursorRay>,
    mut slicer_state: ResMut<SlicerState>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    sliceables_query: Query<(), With<Sliceable>>,
) {
    let Some(cursor_ray) = cursor_ray.0 else {
        return;
    };

    if mouse.pressed(MouseButton::Left) {
        raycast.debug_cast_ray(cursor_ray, &default(), &mut gizmos);
        let hits = raycast.cast_ray(cursor_ray, &default());
        match *slicer_state {
            SlicerState::Idle => {
                if hits.is_empty() {
                    *slicer_state = SlicerState::NoTarget;
                } else {
                    let hit = &hits[0];
                    let sliceable = sliceables_query.get(hit.0);
                    if sliceable.is_ok() {
                        *slicer_state = SlicerState::FirstHit(hits[0].1.position(), hits[0].0);
                    }
                }
            }
            SlicerState::NoTarget => {
                if !hits.is_empty() {
                    let hit = &hits[0];
                    let sliceable = sliceables_query.get(hit.0);
                    if sliceable.is_ok() {
                        *slicer_state = SlicerState::FirstHit(hits[0].1.position(), hits[0].0);
                    }
                }
            }
            SlicerState::FirstHit(pos, entity) => {
                if hits.is_empty() {
                    *slicer_state = SlicerState::NoTarget;
                } else {
                    let hit = &hits[0];
                    if hit.0 == entity && hit.1.position() != pos {
                        *slicer_state = SlicerState::Slicing {
                            start: (pos, entity),
                            last_hit: hits[0].1.position(),
                        };
                    } else if hit.0 != entity {
                        let sliceable = sliceables_query.get(hit.0);
                        if sliceable.is_ok() {
                            *slicer_state = SlicerState::FirstHit(hit.1.position(), hit.0);
                        }
                    }
                }
            }
            SlicerState::Slicing { start, last_hit: _ } => {
                if !hits.is_empty() {
                    let hit = &hits[0];
                    if hit.0 == start.1 && hit.1.position() != start.0 {
                        *slicer_state = SlicerState::Slicing {
                            start,
                            last_hit: hit.1.position(),
                        };
                    } else if hit.0 != start.1 {
                        let sliceable = sliceables_query.get(hit.0);
                        if sliceable.is_ok() {
                            *slicer_state = SlicerState::FirstHit(hit.1.position(), hit.0);
                        }
                    }
                }
            }
        };
    } else {
        match *slicer_state {
            SlicerState::Slicing { start, last_hit } => {
                let slice_event = SliceAttemptEvent {
                    begin: start.0,
                    end: last_hit,
                    entity: start.1,
                };
                commands.trigger(slice_event);
            }
            _ => (),
        }
        *slicer_state = SlicerState::Idle;
    }
}

fn slice(
    trigger: Trigger<SliceAttemptEvent>,
    mut commands: Commands,
    meshes_assets: Res<Assets<Mesh>>,
    mut fragmentation_queue: ResMut<FragmentationQueue>,
    cameras: Query<&mut Transform, With<Camera>>,
    sliceables: Query<
        (&Transform, &GlobalTransform, &Handle<Mesh>),
        (With<Sliceable>, Without<Camera>),
    >,
) {
    let camera_tranform = cameras.single();
    let slice = trigger.event();
    if let Ok((transform, global_transform, mesh_handle)) = sliceables.get(slice.entity) {
        let mesh = meshes_assets.get(mesh_handle).unwrap();

        let inver_trsfrm = global_transform.affine().inverse();
        let local_cam = inver_trsfrm.matrix3 * Vec3A::from(camera_tranform.translation)
            + inver_trsfrm.translation;
        let local_begin =
            inver_trsfrm.matrix3 * Vec3A::from(slice.begin) + inver_trsfrm.translation;
        let local_end = inver_trsfrm.matrix3 * Vec3A::from(slice.end) + inver_trsfrm.translation;
        let local_qr = local_begin - local_cam;
        let local_qs = local_end - local_cam;

        let plane = Plane::new(local_begin, (local_qr.cross(local_qs).normalize()).into());

        if let Some(mesh_fragments) = slice_bevy_mesh(plane, mesh) {
            // commands.spawn((
            //     PbrBundle {
            //         mesh: meshes_assets.add(Plane3d::new(qr.cross(qs))),
            //         transform: Transform::from_translation(event.begin),
            //         material: materials.add(Color::rgb_u8(124, 144, 255)),
            //         ..default()
            //     },
            //     SliceableObject,
            // ));
            // Let other systems react to the slice event with a valid entity
            commands.trigger(SliceEvent {
                entity: slice.entity,
            });
            // Set it as non sliceable
            commands.entity(slice.entity).remove::<Sliceable>();
            // Queue it to be despawned and fragmented
            let fragments_spawn = SpawnFragments {
                sliced_entity: slice.entity,
                slice_positions: (slice.begin, slice.end),
                sliced_object_transform: transform.clone(),
                fragments_meshes: mesh_fragments,
            };
            fragmentation_queue.queue.push((
                fragments_spawn,
                // TODO Hardcode wait duration is hacky. Find a better way
                Timer::new(
                    Duration::from_millis(PLAYER_SLICE_FRAGMENTATION_DELAY_MS),
                    TimerMode::Once,
                ),
            ));
        }
    }
}

fn dequeue_fragmentations(
    mut commands: Commands,
    time: Res<Time>,
    mut fragmentation_queue: ResMut<FragmentationQueue>,
) {
    let mut ready_fragmentations = Vec::new();
    for (index, fragmentation) in fragmentation_queue.queue.iter_mut().enumerate() {
        fragmentation.1.tick(time.delta());
        if fragmentation.1.finished() {
            ready_fragmentations.push(index);
        }
    }
    for i in ready_fragmentations {
        let fragmentation = fragmentation_queue.queue.remove(i); // Cannot swap remove without updating the indexes
        commands.trigger(fragmentation.0);
    }
}

/// Impulse to force the fragments appart, more satisfying
pub const FRAGMENTS_SEPARATION_IMPULSE_FACTOR: f32 = 2500.; // 3000
pub const FRAGMENTS_SLICE_DIRECTION_IMPULSE_FACTOR: f32 = 225000.;
pub const FRAGMENTS_TORQUE_IMPULSE_FACTOR: f32 = 150000000.;

fn fragment_entity(
    trigger: Trigger<SpawnFragments>,
    mut commands: Commands,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    mut meshes_assets: ResMut<Assets<Mesh>>,
    gltf_handles: Res<HandleMap<GltfKey>>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    let fragments_info = trigger.event();

    // Despawn the fragmented entity
    commands.entity(fragments_info.sliced_entity).despawn();

    // Spawn the fragments
    let gltf_handle = &gltf_handles[&GltfKey::Dummy];
    let Some(gltf) = assets_gltf.get(gltf_handle) else {
        return;
    };
    // let Some(gltf_mesh) = assets_gltfmesh.get(&gltf.meshes[0]) else {
    //     return;
    // };
    // let mesh_handle = &gltf_mesh.primitives[0].mesh;
    // TODO Get the mat handle from the sliced entity
    let mat_handle = &gltf.materials[0];

    for mesh in fragments_info.fragments_meshes.iter() {
        let Some(collider) = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::ConvexHull)
        else {
            continue;
        };
        let Some(aabb) = mesh.compute_aabb() else {
            continue;
        };
        let mesh_handle = meshes_assets.add(mesh.clone());
        let frag_entity = commands
            .spawn((
                Name::new("Fragment"),
                StateScoped(Screen::Playing),
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    transform: Transform::from_translation(
                        fragments_info.sliced_object_transform.translation,
                    )
                    .with_scale(Vec3::splat(ASSETS_SCALE)), // TODO Retrive scale of the sliced entity
                    // material: materials.add(Color::srgb_u8(124, 144, 255)),
                    material: mat_handle.clone(),
                    ..default()
                },
                // TODO May make it sliceable again
                // Sliceable,
                // Physics
                RigidBody::Dynamic,
                collider,
                ActiveCollisionTypes::default(),
                Friction::coefficient(0.7),
                Restitution::coefficient(0.05),
                ColliderMassProperties::Density(2.0),
                // Logic
                SlicedFragment::new(),
            ))
            .id();

        let frag_center: Vec3 = aabb.center.into();
        let separating_impulse =
            FRAGMENTS_SEPARATION_IMPULSE_FACTOR * (frag_center - fragments_info.slice_positions.0);

        let slice_direction =
            (fragments_info.slice_positions.1 - fragments_info.slice_positions.0).normalize();
        let slice_direction_impulse = slice_direction * FRAGMENTS_SLICE_DIRECTION_IMPULSE_FACTOR;

        // let torque_impulse = (fragments_info.slice_positions.0 - DEFAULT_GLADIATOR_POS).normalize()
        //     * FRAGMENTS_TORQUE_IMPULSE_FACTOR;
        // fragments_info.sliced_object_transform.right();

        // let dummy_vector = (fragments_info.slice_positions.0 - DEFAULT_GLADIATOR_POS).normalize();

        let torque_impulse =
            fragments_info.sliced_object_transform.right() * FRAGMENTS_TORQUE_IMPULSE_FACTOR;

        commands.entity(frag_entity).insert(ExternalImpulse {
            impulse: separating_impulse + slice_direction_impulse,
            torque_impulse,
        });
    }
}

pub fn despawn_fragments(
    mut commands: Commands,
    time: Res<Time>,
    mut fragments_query: Query<(Entity, &mut SlicedFragment)>,
) {
    for (entity, mut frag) in fragments_query.iter_mut() {
        frag.despawn_timer.tick(time.delta());
        if frag.despawn_timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
