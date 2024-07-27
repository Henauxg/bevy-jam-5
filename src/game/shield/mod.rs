use bevy::app::App;

pub mod scoring;
pub mod spawning;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((spawning::plugin, scoring::plugin));
}
