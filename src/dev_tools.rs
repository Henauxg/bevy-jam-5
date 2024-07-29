//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_rapier3d::render::RapierDebugRenderPlugin;

use crate::{
    game::{
        arena::ArenaMode,
        camera::{display_pan_orbit_camera_state, update_pan_orbit_camera},
        sword::dummies::debug_draw_dummy_slots,
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // app.add_plugins(WorldInspectorPlugin::new());
    // app.add_plugins(RapierDebugRenderPlugin::default());

    app.add_systems(
        Update,
        (
            // Print state transitions in dev builds
            log_transitions::<Screen>,
            log_transitions::<ArenaMode>,
            // Debug rendering
            debug_draw_dummy_slots,
            // Debug camera controls
            update_pan_orbit_camera,
            display_pan_orbit_camera_state.run_if(input_just_pressed(KeyCode::KeyC)),
        ),
    );
}
