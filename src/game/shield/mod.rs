use bevy::{
    app::App,
    prelude::{Commands, OnEnter, OnExit},
};
use camera::SetShieldModeCamera;
use throwers::{SpawnJugThrowers, ThrowersData};

use super::{arena::ArenaMode, spawn::shield::SpawnShield};

pub mod camera;
pub mod collisions;
pub mod player_control;
pub mod throwers;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        camera::plugin,
        throwers::plugin,
        player_control::plugin,
        collisions::plugin,
    ));
    app.add_systems(OnEnter(ArenaMode::Shield), on_enter_shield_mode);
    app.add_systems(OnExit(ArenaMode::Shield), on_exit_shield_mode);
}

pub fn on_enter_shield_mode(mut commands: Commands) {
    commands.insert_resource(ThrowersData::default());
    commands.trigger(SpawnShield);
    commands.trigger(SpawnJugThrowers);
    commands.trigger(SetShieldModeCamera);
}

pub fn on_exit_shield_mode(mut commands: Commands) {
    commands.remove_resource::<ThrowersData>();
}
