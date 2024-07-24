use std::time::Duration;

use bevy::{
    app::{App, Update},
    asset::{Assets, Handle},
    color::Color,
    core::Name,
    input::ButtonInput,
    math::{Vec3, Vec3A},
    pbr::{PbrBundle, StandardMaterial},
    prelude::{
        Camera, Commands, Component, DespawnRecursiveExt, Entity, Event, Gizmos, GlobalTransform,
        IntoSystemConfigs, Mesh, MouseButton, Query, Res, ResMut, Resource, StateScoped, Transform,
        Trigger, With, Without,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
    utils::default,
};
use bevy_ghx_destruction::{slicing::slicing::slice_bevy_mesh, types::Plane};
use bevy_mod_raycast::{cursor::CursorRay, prelude::Raycast};
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, ComputedColliderShape, ExternalImpulse,
    Friction, Restitution, RigidBody,
};

use crate::{screen::Screen, AppSet};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Sliceable>();
    app.register_type::<SliceEvent>();
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
struct SliceEvent {
    begin: Vec3,
    end: Vec3,
    entity: Entity,
}

#[derive(Resource, Debug, Clone, Default, Reflect)]
struct SlicerState {
    begin: Vec3,
    end: Vec3,
}

fn detect_slices(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    cursor_ray: Res<CursorRay>,
    mut slicer_state: ResMut<SlicerState>,
    mut raycast: Raycast,
    mut gizmos: Gizmos,
) {
    let Some(cursor_ray) = cursor_ray.0 else {
        return;
    };

    raycast.debug_cast_ray(cursor_ray, &default(), &mut gizmos);
    let hits = raycast.cast_ray(cursor_ray, &default());

    for (entity, intersection) in hits.iter() {
        let pos = intersection.position();

        if mouse.just_pressed(MouseButton::Left) {
            slicer_state.begin = pos;
        }
        if mouse.just_released(MouseButton::Left) {
            slicer_state.end = pos;
            if slicer_state.begin != slicer_state.end {
                commands.trigger(SliceEvent {
                    begin: slicer_state.begin,
                    end: slicer_state.end,
                    entity: *entity,
                });
            }
        }
    }
}

fn slice(
    trigger: Trigger<SliceEvent>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<&mut Transform, With<Camera>>,
    sliceables: Query<
        (&Transform, &GlobalTransform, &Handle<Mesh>),
        (With<Sliceable>, Without<Camera>),
    >,
    mut meshes_assets: ResMut<Assets<Mesh>>,
) {
    let camera_tranform = cameras.single();
    let event = trigger.event();
    if let Ok((transform, global_transform, mesh_handle)) = sliceables.get(event.entity) {
        let mesh = meshes_assets.get(mesh_handle).unwrap();

        let inver_trsfrm = global_transform.affine().inverse();
        let local_cam = inver_trsfrm.matrix3 * Vec3A::from(camera_tranform.translation)
            + inver_trsfrm.translation;
        let local_begin =
            inver_trsfrm.matrix3 * Vec3A::from(event.begin) + inver_trsfrm.translation;
        let local_end = inver_trsfrm.matrix3 * Vec3A::from(event.end) + inver_trsfrm.translation;
        let local_qr = local_begin - local_cam;
        let local_qs = local_end - local_cam;

        let plane = Plane::new(local_begin, (local_qr.cross(local_qs).normalize()).into());

        // let optional_mesh_fragments =;
        if let Some(mesh_fragments) = slice_bevy_mesh(plane, mesh) {
            commands.entity(event.entity).despawn();

            // commands.spawn((
            //     PbrBundle {
            //         mesh: meshes_assets.add(Plane3d::new(qr.cross(qs))),
            //         transform: Transform::from_translation(event.begin),
            //         material: materials.add(Color::rgb_u8(124, 144, 255)),
            //         ..default()
            //     },
            //     SliceableObject,
            // ));

            spawn_fragments(
                &mesh_fragments,
                &mut materials,
                &mut meshes_assets,
                &mut commands,
                transform.translation,
                local_begin,
            );
        }
    }
}

pub const FRAGMENT_INITIAL_IMPULSE_FACTOR: f32 = 2.;

fn spawn_fragments(
    mesh_fragments: &[Mesh],
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes_assets: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
    sliced_mesh_pos: Vec3,
    slice_plane_point: Vec3A,
) {
    for mesh in mesh_fragments {
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
                Name::new("Dummy fragment"),
                StateScoped(Screen::Playing),
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    transform: Transform::from_translation(sliced_mesh_pos),
                    material: materials.add(Color::srgb_u8(124, 144, 255)),
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
        let frag_center = aabb.center;
        let impulse = (FRAGMENT_INITIAL_IMPULSE_FACTOR * (frag_center - slice_plane_point)).into();
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
