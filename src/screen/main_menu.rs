//! The screen that appears when the game starts.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{
        assets::{FontKey, HandleMap, DEFAULT_FONT_KEY},
        camera::{PanOrbitSettings, PanOrbitState},
    },
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::MainMenu), enter_title);

    app.register_type::<MainMenuAction>();
    app.add_systems(
        Update,
        handle_title_action.run_if(in_state(Screen::MainMenu)),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum MainMenuAction {
    Play,
    // Credits,
    /// Exit doesn't work well with embedded applications.
    #[cfg(not(target_family = "wasm"))]
    Exit,
}

fn enter_title(
    mut commands: Commands,
    // mut fonts: ResMut<Assets<Font>>,
    font_handles: Res<HandleMap<FontKey>>,
    mut camera_query: Query<(&mut PanOrbitState, &mut PanOrbitSettings)>,
) {
    let font = font_handles.get(&DEFAULT_FONT_KEY).unwrap().clone();
    // Seems to need additonal setings. Spacing is not right
    // let new_default_font = fonts
    //     .get(font_handles.get(&FontKey::Augustus).unwrap())
    //     .unwrap()
    //     .clone();
    // fonts.insert(&Handle::default(), new_default_font);

    commands
        .bottom_ui_root()
        .insert(StateScoped(Screen::MainMenu))
        .with_children(|children| {
            children.button("Play", font).insert(MainMenuAction::Play);
            // children.button("Credits").insert(TitleAction::Credits);

            // #[cfg(not(target_family = "wasm"))]
            // children.button("Exit").insert(TitleAction::Exit);
        });

    // Setup camera
    let Ok((mut cam_state, mut cam_settings)) = camera_query.get_single_mut() else {
        return;
    };
    cam_state.center = Vec3::ZERO;
    cam_state.radius = 51.71328;
    cam_state.pitch = -0.3002041;
    cam_state.yaw = 0.5580911;

    cam_settings.auto_orbit = true;
}

fn handle_title_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&MainMenuAction>,
    #[cfg(not(target_family = "wasm"))] mut app_exit: EventWriter<AppExit>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                MainMenuAction::Play => next_screen.set(Screen::Playing),
                // TitleAction::Credits => next_screen.set(Screen::Credits),
                #[cfg(not(target_family = "wasm"))]
                MainMenuAction::Exit => {
                    app_exit.send(AppExit::Success);
                }
            }
        }
    }
}
