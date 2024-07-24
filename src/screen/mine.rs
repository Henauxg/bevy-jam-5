use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::Screen;
use crate::game::spawn::mine_scene::SpawnMineScene;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Mine), enter_mine);
    app.add_systems(OnExit(Screen::Mine), exit_mine);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Mine).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn enter_mine(mut commands: Commands) {
    commands.trigger(SpawnMineScene);
    // commands.trigger(PlaySoundtrack::Key(SoundtrackKey::Excavation));
}

fn exit_mine(mut _commands: Commands) {
    // We could use [`StateScoped`] on the sound playing entities instead.
    // commands.trigger(PlaySoundtrack::Disable);
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
