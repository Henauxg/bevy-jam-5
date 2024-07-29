mod loading;
mod main_menu;
mod playing;

use bevy::prelude::*;

use crate::game::arena::ArenaMode;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();
    app.enable_state_scoped_entities::<ArenaMode>();

    app.add_plugins((loading::plugin, main_menu::plugin, playing::plugin));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    #[default]
    Loading,
    MainMenu,
    Playing,
}
