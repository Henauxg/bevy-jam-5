//! Plays animations from a skinned glTF.

use std::time::Duration;

use bevy::{animation::animate_targets, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Loading),
        setup_player_animations.before(animate_targets),
    );
    app.add_systems(Update, attach_player_animations);
}

use crate::screen::Screen;

use super::assets::{AnimationKey, HandleMap};
#[derive(Resource)]
pub struct PlayerAnimations {
    walk_anim: AnimationNodeIndex,
    idle_anim: AnimationNodeIndex,
    slash_anim: AnimationNodeIndex,
    graph: Handle<AnimationGraph>,
}

fn setup_player_animations(
    mut commands: Commands,
    anim_handles: Res<HandleMap<AnimationKey>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut graph = AnimationGraph::new();
    let idle_anim = graph.add_clip(
        anim_handles[&AnimationKey::GladiatorIdle].clone_weak(),
        1.0,
        graph.root,
    );
    let walk_anim = graph.add_clip(
        anim_handles[&AnimationKey::GladiatorWalk].clone_weak(),
        1.0,
        graph.root,
    );
    let slash_anim = graph.add_clip(
        anim_handles[&AnimationKey::GladiatorSlash].clone_weak(),
        1.0,
        graph.root,
    );
    let graph = graphs.add(graph);

    commands.insert_resource(PlayerAnimations {
        idle_anim,
        walk_anim,
        slash_anim,
        graph: graph.clone(),
    });
}

fn attach_player_animations(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), (Added<AnimationPlayer>)>, // TODOD With<Player>, .. But only the scene root has the Player marker, not the animation player entity
    // anim_handles: Res<HandleMap<AnimationKey>>,
    animations: Res<PlayerAnimations>,
    // mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // info!("attach_player_animations ");
    for (entity, mut animation_player) in &mut players {
        info!("attach_player_animations on entity {:?}", entity);
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut animation_player, animations.idle_anim, Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}
