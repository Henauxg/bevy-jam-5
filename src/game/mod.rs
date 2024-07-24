//! Game mechanics and content.

use bevy::prelude::*;

mod animation;
pub mod arena;
pub mod assets;
pub mod audio;
pub mod dummies;
mod movement;
pub mod spawn;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        // animation::plugin,

        // movement::plugin,
        audio::plugin,
        assets::plugin,
        spawn::plugin,
        dummies::plugin,
        arena::plugin,
    ));
}
