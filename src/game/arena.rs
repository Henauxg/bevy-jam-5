use bevy::{
    app::App,
    prelude::{AppExtStates, StateSet, SubStates},
    reflect::Reflect,
};
use rand::{distributions::Standard, prelude::Distribution, Rng};

use crate::screen::Screen;

use super::assets::ImageKey;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ArenaMode>();
    app.add_sub_state::<ArenaMode>();
}

#[derive(SubStates, Default, Reflect, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(Screen = Screen::Playing)]
pub enum ArenaMode {
    #[default]
    None,
    Sword,
    Shield,
    GameOver,
}
impl ArenaMode {
    pub fn to_image_key(&self) -> ImageKey {
        match self {
            ArenaMode::Sword => ImageKey::Sword,
            ArenaMode::Shield => ImageKey::Shield,
            // TODO
            ArenaMode::None => ImageKey::Sword,
            ArenaMode::GameOver => ImageKey::Sword,
        }
    }
}

impl Distribution<ArenaMode> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ArenaMode {
        match rng.gen_range(0..=1) {
            0 => ArenaMode::Sword,
            _ => ArenaMode::Shield,
        }
    }
}
