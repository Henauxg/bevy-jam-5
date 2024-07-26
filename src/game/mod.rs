//! Game mechanics and content.

use bevy::prelude::*;

pub mod arena;
pub mod assets;
pub mod audio;
pub mod dummies;
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
