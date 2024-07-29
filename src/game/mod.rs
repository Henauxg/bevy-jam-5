//! Game mechanics and content.

use bevy::prelude::*;

pub mod arena;
pub mod assets;
pub mod audio;
pub mod camera;
pub mod cycle;
pub mod player_animation;
pub mod score;
pub mod shattering;
pub mod shield;
pub mod spawn;
pub mod sword;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        assets::plugin,
        spawn::plugin,
        sword::plugin,
        shield::plugin,
        arena::plugin,
        score::plugin,
        player_animation::plugin,
        shattering::plugin,
        cycle::plugin,
    ));
}
