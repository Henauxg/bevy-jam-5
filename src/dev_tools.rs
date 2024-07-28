//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    game::{arena::ArenaMode, sword::dummies::debug_draw_dummy_slots},
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(WorldInspectorPlugin::new());

    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(Update, log_transitions::<ArenaMode>);

    app.add_systems(Update, debug_draw_dummy_slots);
}
