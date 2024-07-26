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

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Sliceable>();
    app.register_type::<SliceAttemptEvent>();
    app.register_type::<SlicedEvent>();
    app.register_type::<SlicerState>();

    app.add_systems(
        Update,
        (
            detect_slices.in_set(AppSet::RecordInput),
            despawn_fragments.in_set(AppSet::Update),
        ),
    );
    app.init_resource::<SlicerState>();

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
pub struct SliceAttemptEvent {
    pub begin: Vec3,
    pub end: Vec3,
    pub entity: Entity,
}

#[derive(Event, Debug, Clone, Reflect)]
/// An entity has been sliced
pub struct SlicedEvent {
    pub entity: Entity,
}

#[derive(Event, Debug, Clone, Reflect)]
struct SpawnFragments {
    sliced_entity: Entity,
    sliced_transform: Transform,
    mesh_fragments: [Mesh; 2],
    slice_start_pos: Vec3,
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

    cameras: Query<&mut Transform, With<Camera>>,
    sliceables: Query<
        (&Transform, &GlobalTransform, &Handle<Mesh>),
        (With<Sliceable>, Without<Camera>),
    >,
    meshes_assets: Res<Assets<Mesh>>,
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
            commands.trigger(SlicedEvent {
                entity: slice.entity,
            });
            // Then despawn it
            commands.trigger(SpawnFragments {
                sliced_entity: slice.entity,
                slice_start_pos: slice.begin,
                sliced_transform: transform.clone(),
                mesh_fragments,
            });
        }
    }
}

/// Impulse to force the fragments appart, more satisfying
pub const FRAGMENT_INITIAL_IMPULSE_FACTOR: f32 = 3000.;

fn fragment_entity(
    trigger: Trigger<SpawnFragments>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    for mesh in fragments_info.mesh_fragments.iter() {
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
                        fragments_info.sliced_transform.translation,
                    )
                    .with_scale(Vec3::splat(ASSETS_SCALE)), // TODO Retrive scale of the sliced entity
                    // material: materials.add(Color::srgb_u8(124, 144, 255)),
                    material: mat_handle.clone(),
                    ..default()
                },
                // Sliceable,
                // Wireframe,
                // WireframeColor {
                //     color: Color::Srgba(GREEN),
                // },
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
        let impulse =
            FRAGMENT_INITIAL_IMPULSE_FACTOR * (frag_center - fragments_info.slice_start_pos);
        commands.entity(frag_entity).insert(ExternalImpulse {
            impulse,
            torque_impulse: Vec3::ZERO,
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
