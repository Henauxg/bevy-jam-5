//! Spawn the main level by triggering other observers.

use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{ORANGE_RED, SANDY_BROWN, WHITE},
    prelude::*,
};
use bevy_rapier3d::prelude::{ActiveCollisionTypes, Collider, Friction, Restitution};

use crate::game::assets::{HandleMap, SceneKey, ARENA_SCALE};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_arena);
}

pub const DEFAULT_GLADIATOR_POS: Vec3 = Vec3::new(0., 0., 0.0);
pub const DEFAULT_GLADIATOR_LOOK_AT: Vec3 = Vec3::new(0., 0., 1.5);

pub const GROUND_FRICTION: f32 = 1.;

#[derive(Event, Debug)]
pub struct SpawnArena;

fn spawn_arena(
    _trigger: Trigger<SpawnArena>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scenes_handles: Res<HandleMap<SceneKey>>,
) {
    // Scene lights
    commands.insert_resource(AmbientLight {
        color: Color::Srgba(WHITE),
        brightness: 45.,
    });
    commands.spawn((
        Name::new("DirectionalLight"),
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 6000.,
                color: Color::srgb(1.0, 0.85, 0.65),
                // color: bevy::prelude::Color::Srgba(ORANGE_RED),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(5.0, 10.0, 2.0),
                // rotation: Quat::from_euler(EulerRot::ZYX, 0., -2. * PI / 3., -2. * PI / 3.),
                rotation: Quat::from_euler(EulerRot::ZYX, 0., -3. * PI / 3., -1. * PI / 3.),
                ..default()
            },
            ..default()
        },
    ));
    // Only 1 dir light allowed for wasm builds
    // commands.spawn((DirectionalLightBundle {
    //     directional_light: DirectionalLight {
    //         shadows_enabled: false,
    //         illuminance: 2000.,
    //         color: Color::Srgba(ORANGE_RED),
    //         ..default()
    //     },
    //     transform: Transform {
    //         translation: Vec3::new(5.0, 10.0, 2.0),
    //         rotation: Quat::from_euler(EulerRot::ZYX, 0., PI * 4. / 5., -PI / 3.),
    //         ..default()
    //     },
    //     ..default()
    // },));

    // Prototype ground
    let radius = 1500.;
    let height = 20.;
    commands.spawn((
        Name::new("Ground"),
        PbrBundle {
            mesh: meshes.add(Cylinder::new(radius, height)),
            material: materials.add(StandardMaterial {
                base_color: Color::Srgba(SANDY_BROWN),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -height / 2., 0.0),
            ..default()
        },
        Collider::cylinder(height / 2., radius),
        (ActiveCollisionTypes::default()),
        Friction::coefficient(GROUND_FRICTION),
        Restitution::coefficient(0.0),
    ));

    // commands.spawn((
    //     Name::new("Ground details"),
    //     SceneBundle {
    //         scene: scenes_handles[&SceneKey::GroundDetails].clone_weak(),
    //         transform: Transform::from_translation(Vec3::ZERO)
    //             .with_scale(Vec3::splat(ASSETS_SCALE)),
    //         ..default()
    //     },
    // ));

    // Arena
    commands.spawn((
        Name::new("Arena"),
        SceneBundle {
            scene: scenes_handles[&SceneKey::Arena].clone_weak(),
            transform: Transform::from_translation(Vec3::ZERO)
                // .looking_at(spawn_info.looking_at, Vec3::Y)
                .with_scale(Vec3::splat(ARENA_SCALE)),
            ..default()
        },
    ));
}
