use bevy::prelude::*;

use crate::{
    game::{
        assets::{HandleMap, SceneKey},
        player_animation::{EquipmentSlot, EquipmentToAttach},
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_helmet);
    app.register_type::<Helmet>();
}

#[derive(Event, Debug)]
pub struct SpawnHelmet;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct Helmet;

fn spawn_helmet(
    _trigger: Trigger<SpawnHelmet>,
    mut commands: Commands,
    scenes_handles: Res<HandleMap<SceneKey>>,
) {
    commands.spawn((
        Name::new("Helmet"),
        StateScoped(Screen::Playing),
        SceneBundle {
            scene: scenes_handles[&SceneKey::Helmet].clone_weak(),
            transform: Transform::IDENTITY,
            ..default()
        },
        Helmet,
        EquipmentToAttach {
            slot: EquipmentSlot::Head,
        },
    ));
}
