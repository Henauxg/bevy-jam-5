//! Handles spawning of entities. Here, we are using
//! [observers](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Observer.html)
//! for this, but you could also use `Events<E>` or `Commands`.

use bevy::prelude::*;

pub mod arena;
pub mod dummy;
pub mod mine_scene;
pub mod player;
pub mod rock;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        arena::plugin,
        player::plugin,
        mine_scene::plugin,
        rock::plugin,
        dummy::plugin,
    ));
}
