use bevy::{
    app::App,
    prelude::{Res, Resource},
    reflect::Reflect,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ArenaMode>();
}

#[derive(Resource, Default, Clone, PartialEq, Eq, Reflect)]
pub enum ArenaMode {
    #[default]
    None,
    Dummies,
}

pub fn arena_is_in_dummies_mode(mode: Res<ArenaMode>) -> bool {
    *mode == ArenaMode::Dummies
}
