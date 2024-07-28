//! The title screen that appears when the game starts.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{
        assets::{FontKey, HandleMap},
        camera::{PanOrbitSettings, PanOrbitState},
        spawn::arena::SpawnArena,
    },
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), enter_title);

    app.register_type::<TitleAction>();
    app.add_systems(Update, handle_title_action.run_if(in_state(Screen::Title)));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum TitleAction {
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
    commands.trigger(SpawnArena);

    let font = font_handles.get(&FontKey::RomanSD).unwrap().clone();
    // Seems to need additonal setings. Spacing is not right
    // let new_default_font = fonts
    //     .get(font_handles.get(&FontKey::Augustus).unwrap())
    //     .unwrap()
    //     .clone();
    // fonts.insert(&Handle::default(), new_default_font);

    commands
        .bottom_ui_root()
        .insert(StateScoped(Screen::Title))
        .with_children(|children| {
            children.button("Play", font).insert(TitleAction::Play);
            // children.button("Credits").insert(TitleAction::Credits);

            // #[cfg(not(target_family = "wasm"))]
            // children.button("Exit").insert(TitleAction::Exit);
        });

    // Setup camera
    let Ok((mut cam_state, mut cam_settings)) = camera_query.get_single_mut() else {
        return;
    };
    cam_state.center = Vec3::ZERO;
    cam_state.radius = 25.68;
    cam_state.pitch = -0.28100345;
    cam_state.yaw = -0.7212212;

    cam_settings.auto_orbit = true;
}

fn handle_title_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&TitleAction>,
    #[cfg(not(target_family = "wasm"))] mut app_exit: EventWriter<AppExit>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                TitleAction::Play => next_screen.set(Screen::Playing),
                // TitleAction::Credits => next_screen.set(Screen::Credits),
                #[cfg(not(target_family = "wasm"))]
                TitleAction::Exit => {
                    app_exit.send(AppExit::Success);
                }
            }
        }
    }
}
