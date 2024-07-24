#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod screen;
mod ui;

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_ghx_utils::camera::{
    display_pan_orbit_camera_state, update_pan_orbit_camera, PanOrbitCameraBundle,
    PanOrbitSettings, PanOrbitState,
};
use bevy_mod_raycast::cursor::CursorRayPlugin;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "bevy-jam-5".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.2),
                    },
                    ..default()
                }),
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (
                update_pan_orbit_camera,
                display_pan_orbit_camera_state.run_if(input_just_pressed(KeyCode::KeyC)),
            ),
        );

        // Add other plugins.
        app.add_plugins((
            CursorRayPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
        ));
        app.add_plugins((game::plugin, screen::plugin, ui::plugin));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn spawn_camera(mut commands: Commands) {
    // Camera
    let camera_position = Vec3::new(0., 1.5, -5.);
    let look_target = Vec3::ZERO;
    commands.spawn((
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
        PanOrbitCameraBundle {
            camera: Camera3dBundle {
                transform: Transform::from_translation(camera_position)
                    .looking_at(look_target, Vec3::Y),
                ..default()
            },
            state: PanOrbitState {
                center: Vec3::new(-0.2976082, 0.45186156, 0.6121746),
                radius: 5.379136,
                upside_down: false,
                pitch: -0.28798094,
                yaw: -2.6529098,
                ..Default::default()
            },
            settings: PanOrbitSettings {
                auto_orbit: false,
                ..Default::default()
            },
        },
    ));
}
