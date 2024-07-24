//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};

use crate::{game::dummies::spawning::debug_draw_dummy_slots, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);

    app.add_systems(Update, debug_draw_dummy_slots);
}
