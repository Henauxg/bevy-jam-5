use bevy::app::App;

pub mod scoring;
pub mod slicing;
pub mod spawning;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((slicing::plugin, spawning::plugin, scoring::plugin));
}
