use bevy::{
    app::App,
    math::Vec3,
    prelude::{Event, Query, Trigger},
};

use crate::game::camera::{PanOrbitSettings, PanOrbitState};

#[derive(Event, Debug)]
pub struct SetShieldModeCamera;

pub(super) fn plugin(app: &mut App) {
    app.observe(setup_camera);
}

pub fn setup_camera(
    _trigger: Trigger<SetShieldModeCamera>,
    mut camera_query: Query<(&mut PanOrbitState, &mut PanOrbitSettings)>,
) {
    let Ok((mut cam_state, mut cam_settings)) = camera_query.get_single_mut() else {
        return;
    };
    cam_state.center = Vec3::new(0.38106346, -4.2590923, -3.7643096);
    cam_state.radius = 60.033787;
    cam_state.pitch = -0.6771932;
    cam_state.yaw = -3.124136;
    cam_state.needs_transform_refresh = true;

    cam_settings.auto_orbit = false;
}

// TODO handle cam movements with mouse
