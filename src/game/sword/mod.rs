use bevy::{
    app::App,
    prelude::{Commands, OnEnter, OnExit},
};
use dummies::{DummiesData, SpawnDummySlots};

use super::{arena::ArenaMode, spawn::sword::SpawnSword};

pub mod dummies;
pub mod scoring;
pub mod slicing;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((slicing::plugin, dummies::plugin, scoring::plugin));
    app.add_systems(OnEnter(ArenaMode::Sword), on_enter_sword_mode);
    app.add_systems(OnExit(ArenaMode::Sword), on_exit_sword_mode);
}

pub fn on_enter_sword_mode(mut commands: Commands) {
    commands.insert_resource(DummiesData::default());
    commands.trigger(SpawnSword);
    commands.trigger(SpawnDummySlots);
}

pub fn on_exit_sword_mode(mut commands: Commands) {
    commands.remove_resource::<DummiesData>();
}
