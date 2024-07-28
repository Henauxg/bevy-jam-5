use bevy::prelude::*;

use crate::game::{
    arena::ArenaMode,
    assets::{HandleMap, SceneKey},
    player_animation::{EquipmentSlot, EquipmentToAttach},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_shield);
    app.register_type::<Shield>();
}

#[derive(Event, Debug)]
pub struct SpawnShield;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct Shield;

fn spawn_shield(
    _trigger: Trigger<SpawnShield>,
    mut commands: Commands,
    scenes_handles: Res<HandleMap<SceneKey>>,
) {
    commands.spawn((
        Name::new("Shield"),
        StateScoped(ArenaMode::Shield),
        SceneBundle {
            scene: scenes_handles[&SceneKey::Shield].clone_weak(),
            transform: Transform::IDENTITY,
            ..default()
        },
        Shield,
        EquipmentToAttach {
            slot: EquipmentSlot::RightHand, // TODO Use left hand once slot is there
        },
    ));
}
