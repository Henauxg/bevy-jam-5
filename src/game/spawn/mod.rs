//! Handles spawning of entities. Here, we are using
//! [observers](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Observer.html)
//! for this, but you could also use `Events<E>` or `Commands`.

use bevy::prelude::*;

pub mod arena;
pub mod dummy;
pub mod helmet;
pub mod jug;
pub mod jug_thrower;
pub mod player;
pub mod shield;
pub mod sword;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        arena::plugin,
        player::plugin,
        sword::plugin,
        shield::plugin,
        helmet::plugin,
        dummy::plugin,
        jug_thrower::plugin,
        jug::plugin,
    ));
}
