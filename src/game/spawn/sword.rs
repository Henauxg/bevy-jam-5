use bevy::prelude::*;

use crate::{
    game::{
        assets::{HandleMap, SceneKey},
        player_animation::{EquipmentSlot, EquipmentToAttach},
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_sword);
    app.register_type::<Sword>();
}

#[derive(Event, Debug)]
pub struct SpawnSword;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct Sword;

fn spawn_sword(
    _trigger: Trigger<SpawnSword>,
    mut commands: Commands,
    scenes_handles: Res<HandleMap<SceneKey>>,
) {
    commands.spawn((
        Name::new("Sword"),
        StateScoped(Screen::Playing),
        SceneBundle {
            scene: scenes_handles[&SceneKey::Sword].clone_weak(),
            transform: Transform::IDENTITY,
            ..default()
        },
        Sword,
        EquipmentToAttach {
            slot: EquipmentSlot::RightHand,
        },
    ));
}
