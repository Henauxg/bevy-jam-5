use bevy::prelude::*;

use crate::game::{
    arena::ArenaMode,
    assets::{HandleMap, SceneKey},
    player_animation::{EquipmentSlot, EquipmentToAttach},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_sword);
    app.register_type::<Sword>();
}

#[derive(Event, Debug)]
pub struct SpawnSword {
    pub scope: ArenaMode,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub struct Sword;

fn spawn_sword(
    trigger: Trigger<SpawnSword>,
    mut commands: Commands,
    scenes_handles: Res<HandleMap<SceneKey>>,
) {
    commands.spawn((
        Name::new("Sword"),
        StateScoped(trigger.event().scope),
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
