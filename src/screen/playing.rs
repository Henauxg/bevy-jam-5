use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::Screen;
use crate::game::{
    arena::ArenaMode,
    spawn::{
        arena::{DEFAULT_GLADIATOR_LOOK_AT, DEFAULT_GLADIATOR_POS},
        player::SpawnPlayer,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn enter_playing(mut commands: Commands, mut next_arena_mode: ResMut<NextState<ArenaMode>>) {
    commands.trigger(SpawnPlayer {
        pos: DEFAULT_GLADIATOR_POS,
        looking_at: DEFAULT_GLADIATOR_LOOK_AT,
    });

    let random_arena_mode: ArenaMode = rand::random();
    next_arena_mode.set(random_arena_mode);

    // commands.trigger(PlaySoundtrack::Key(SoundtrackKey::Gameplay));
}

fn exit_playing(mut _commands: Commands) {
    // We could use [`StateScoped`] on the sound playing entities instead.
    // commands.trigger(PlaySoundtrack::Disable);
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::MainMenu);
}
