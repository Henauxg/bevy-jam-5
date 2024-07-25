//! Game mechanics and content.

use bevy::prelude::*;

mod animation2d;
pub mod arena;
pub mod assets;
pub mod audio;
pub mod dummies;
mod movement;
pub mod player_animation;
pub mod score;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        assets::plugin,
        spawn::plugin,
        dummies::plugin,
        arena::plugin,
        score::plugin,
        player_animation::plugin,
    ));
}
