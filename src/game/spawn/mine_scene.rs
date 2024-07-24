use bevy::{color::palettes::css::WHITE, prelude::*};

use super::rock::SpawnRock;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_mine);
}

#[derive(Event, Debug)]
pub struct SpawnMineScene;

fn spawn_mine(_trigger: Trigger<SpawnMineScene>, mut commands: Commands) {
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
    info!("spawn_mine");
    commands.trigger(SpawnRock);

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            color: bevy::prelude::Color::Srgba(WHITE),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.25, 4.0),
        ..default()
    });
}
