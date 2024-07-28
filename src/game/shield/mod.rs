use bevy::{
    app::App,
    prelude::{Commands, OnEnter, OnExit},
};

use super::{arena::ArenaMode, spawn::shield::SpawnShield};

// pub mod scoring
pub mod camera;
pub mod spawning;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((spawning::plugin, camera::plugin));
    app.add_systems(OnEnter(ArenaMode::Shield), on_enter_sword_mode);
    app.add_systems(OnExit(ArenaMode::Shield), on_exit_sword_mode);
}

pub fn on_enter_sword_mode(mut commands: Commands) {
    // commands.insert_resource(DummiesData::default());
    commands.trigger(SpawnShield);
    // commands.trigger(SpawnDummySlots);
}

pub fn on_exit_sword_mode(mut commands: Commands) {
    // commands.remove_resource::<DummiesData>();
}
