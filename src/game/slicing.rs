use bevy::{
    app::{App, Update},
    asset::{Assets, Handle},
    color::{palettes::css::GREEN, Color},
    input::ButtonInput,
    math::{Vec3, Vec3A},
    pbr::{
        wireframe::{Wireframe, WireframeColor},
        PbrBundle, StandardMaterial,
    },
    prelude::{
        Camera, Commands, Component, Entity, Event, Gizmos, GlobalTransform, IntoSystemConfigs,
        Mesh, MouseButton, Query, Res, ResMut, Resource, Transform, Trigger, With, Without,
    },
    reflect::Reflect,
    utils::default,
};
use bevy_ghx_destruction::{slicing::slicing::slice_bevy_mesh, types::Plane};
use bevy_mod_raycast::{cursor::CursorRay, prelude::Raycast};
use bevy_rapier3d::prelude::{
    ActiveCollisionTypes, Collider, ColliderMassProperties, ComputedColliderShape, Friction,
    Restitution, RigidBody,
};

use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Sliceable>();
    app.register_type::<SliceEvent>();
    app.register_type::<SlicerState>();

    app.add_systems(Update, detect_slices.in_set(AppSet::RecordInput));
    app.init_resource::<SlicerState>();

    app.observe(slice);
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
pub struct Sliceable;

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
    mut raycast: Raycast,
    mut gizmos: Gizmos,
    // mut spawn_sliceable_events: EventWriter<SliceEvent>,
    mut slicer_state: ResMut<SlicerState>,
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

    // TODO viewport_to_world ?
    // let ray = ca

    // let ray_pos = Vec3::new(1.0, 2.0, 3.0);
    // let ray_dir = Vec3::new(0.0, 1.0, 0.0);
    // let max_toi = 4.0;
    // let solid = true;
    // let filter = QueryFilter::default();

    // if let Some((entity, toi)) = rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter) {
    //     // The first collider hit has the entity `entity` and it hit after
    //     // the ray travelled a distance equal to `ray_dir * toi`.
    //     let hit_point = ray_pos + ray_dir * toi;
    //     println!("Entity {:?} hit at point {}", entity, hit_point);
    // }
}

fn slice(
    trigger: Trigger<SliceEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<&mut Transform, With<Camera>>,
    sliceables: Query<
        (&Transform, &GlobalTransform, &Handle<Mesh>),
        (With<Sliceable>, Without<Camera>),
    >,
    mut meshes_assets: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    // mut spawn_sliceable_events: EventReader<SliceEvent>,
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

        let meshes = slice_bevy_mesh(plane, &mesh);

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

        let slice_center = Vec3A::from(transform.translation);
        spawn_fragment(
            meshes,
            &mut materials,
            &mut meshes_assets,
            &mut commands,
            slice_center,
        );
    }
}

fn spawn_fragment(
    meshes: Vec<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes_assets: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
    pos: Vec3A,
) {
    for mesh in meshes {
        let mesh_handle = meshes_assets.add(mesh.clone());
        let Some(collider) = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::ConvexHull)
        else {
            continue;
        };
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, pos.z),
                material: materials.add(Color::srgb_u8(124, 144, 255)),
                ..default()
            },
            Sliceable,
            Wireframe,
            WireframeColor {
                color: Color::Srgba(GREEN),
            },
            RigidBody::Dynamic,
            collider,
            ActiveCollisionTypes::default(),
            Friction::coefficient(0.7),
            Restitution::coefficient(0.05),
            ColliderMassProperties::Density(2.0),
        ));
    }
}
