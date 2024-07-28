use bevy::{
    app::App,
    math::Vec3,
    prelude::{Event, Query, Trigger},
};

use crate::game::camera::{PanOrbitSettings, PanOrbitState};

#[derive(Event, Debug)]
pub struct SetSwordModeCamera;

pub(super) fn plugin(app: &mut App) {
    app.observe(setup_camera);
}

pub fn setup_camera(
    _trigger: Trigger<SetSwordModeCamera>,
    mut camera_query: Query<(&mut PanOrbitState, &mut PanOrbitSettings)>,
) {
    let Ok((mut cam_state, mut cam_settings)) = camera_query.get_single_mut() else {
        return;
    };
    cam_state.center = Vec3::ZERO;
    cam_state.radius = 12.;
    cam_state.pitch = -0.6527581;
    cam_state.yaw = -2.9827762;
    cam_state.needs_transform_refresh = true;

    cam_settings.auto_orbit = false;
}

// TODO handle cam movements with mouse
