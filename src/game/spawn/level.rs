//! Spawn the main level by triggering other observers.

use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{ORANGE_RED, SANDY_BROWN},
    prelude::*,
};
use bevy_rapier3d::prelude::{ActiveCollisionTypes, Collider, Friction, Restitution};

use crate::screen::Screen;

use super::{dummy::SpawnDummy, player::SpawnPlayer};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    commands.trigger(SpawnPlayer {
        pos: Vec3::new(0., 0., -1.5),
        // TODO Why do I need to invert the look at Z
        looking_at: Vec3::new(0., 0., -3.),
    });

    commands.trigger(SpawnDummy { count: 3 });

    // Scene lights
    commands.insert_resource(AmbientLight {
        color: Color::Srgba(ORANGE_RED),
        brightness: 0.05,
    });
    commands.spawn((
        StateScoped(Screen::Playing),
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 4000.,
                color: Color::srgb(1.0, 0.85, 0.65),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(5.0, 10.0, 2.0),
                rotation: Quat::from_euler(EulerRot::ZYX, 0., -PI / 5., -PI / 3.),
                ..default()
            },
            ..default()
        },
    ));
    commands.spawn((
        StateScoped(Screen::Playing),
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: false,
                illuminance: 2000.,
                color: Color::Srgba(ORANGE_RED),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(5.0, 10.0, 2.0),
                rotation: Quat::from_euler(EulerRot::ZYX, 0., PI * 4. / 5., -PI / 3.),
                ..default()
            },
            ..default()
        },
    ));

    // Prototype ground
    let radius = 2000.;
    let height = 20.;
    commands.spawn((
        Name::new("Ground"),
        StateScoped(Screen::Playing),
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
        Friction::coefficient(0.7),
        Restitution::coefficient(0.0),
    ));
}
