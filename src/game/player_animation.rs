//! Plays animations from a skinned glTF.

use std::time::Duration;

use bevy::{animation::animate_targets, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<EquipmentSlot>();
    app.register_type::<EquipmentToAttach>();
    app.add_systems(
        OnEnter(Screen::Loading),
        setup_player_animations.before(animate_targets),
    );
    app.add_systems(
        Update,
        (attach_player_animations, attach_equipments, return_to_idle),
    );
    app.observe(play_slash_animation);
    app.observe(look_towards_sliced_dummy);
}

use crate::screen::Screen;

use super::{
    assets::{AnimationKey, HandleMap},
    spawn::player::Player,
    sword::slicing::SliceEvent,
};

pub const RIGHT_HAND_SLOT: &str = "EquipmentHandle.R";
pub const LEFT_HAND_SLOT: &str = "EquipmentHandle.L";

pub const PLAYER_SLASH_ANIMATION_SPEED: f32 = 3.4;

#[derive(Resource)]
pub struct PlayerAnimations {
    _walk_anim: AnimationNodeIndex,
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
        _walk_anim: walk_anim,
        slash_anim,
        graph: graph.clone(),
    });
}

// TODO With<Player>, .. But only the scene root has the Player marker, not the animation player entity
fn attach_player_animations(
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    animations: Res<PlayerAnimations>,
) {
    for (entity, mut animation_player) in players.iter_mut() {
        // info!("attach_player_animations on entity {:?}", entity);
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

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub enum EquipmentSlot {
    RightHand,
    LeftHand,
}

impl EquipmentSlot {
    pub fn to_name(&self) -> &str {
        match self {
            EquipmentSlot::RightHand => RIGHT_HAND_SLOT,
            EquipmentSlot::LeftHand => LEFT_HAND_SLOT,
        }
    }
}

#[derive(Component, Clone, Reflect)]
pub struct EquipmentToAttach {
    pub slot: EquipmentSlot,
}

fn attach_equipments(
    mut commands: Commands,
    players_query: Query<Entity, With<Player>>,
    names_query: Query<&Name>,
    children_query: Query<&Children>,
    equipments_to_attach_query: Query<(Entity, &EquipmentToAttach)>,
) {
    let Ok(player_entity) = players_query.get_single() else {
        return;
    };

    for (equipment_entity, equipment) in equipments_to_attach_query.iter() {
        let attach_point_entity = find_child_with_name_containing(
            player_entity,
            equipment.slot.to_name(),
            &names_query,
            &children_query,
        );

        if let Some(parent_entity) = attach_point_entity {
            commands.entity(equipment_entity).set_parent(parent_entity);
            commands
                .entity(equipment_entity)
                .remove::<EquipmentToAttach>();
        }
    }
}

pub fn find_child_with_name_containing(
    root_entity: Entity,
    name_to_match: &str,
    names_query: &Query<&Name>,
    children_query: &Query<&Children>,
) -> Option<Entity> {
    let mut queue = Vec::new();
    queue.push(root_entity);
    while let Some(current_entity) = queue.pop() {
        let name = names_query.get(current_entity);
        if let Ok(name) = name {
            if name.as_str() == name_to_match {
                return Some(current_entity);
            }
        }
        let children = children_query.get(current_entity);
        if let Ok(children) = children {
            queue.extend(children);
        }
    }
    None
}

// TODO With<Player>, .. But only the scene root has the Player marker, not the animation player entity
fn play_slash_animation(
    _trigger: Trigger<SliceEvent>,
    animations: Res<PlayerAnimations>,
    mut players_query: Query<(Entity, &mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (_entity, mut animation_player, mut transitions) in players_query.iter_mut() {
        // info!("Transition to slash animation");
        transitions
            .play(
                &mut animation_player,
                animations.slash_anim,
                Duration::from_millis(50),
            )
            .set_speed(PLAYER_SLASH_ANIMATION_SPEED);
        // TODO Getting the animation duration here would be great
    }
}

fn look_towards_sliced_dummy(
    trigger: Trigger<SliceEvent>,
    mut players_query: Query<&mut Transform, With<Player>>,
    transforms: Query<&mut Transform, Without<Player>>,
) {
    let sliced_entity = trigger.event().entity;
    let Ok(sliced_pos) = transforms.get(sliced_entity) else {
        return;
    };

    for mut transform in players_query.iter_mut() {
        transform.look_at(sliced_pos.translation, Vec3::Y);
    }
}

// TODO With<Player>, .. But only the scene root has the Player marker, not the animation player entity
fn return_to_idle(
    animations: Res<PlayerAnimations>,
    mut players_query: Query<(Entity, &mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (_entity, mut animation_player, mut transitions) in players_query.iter_mut() {
        let Some((_, anim)) = animation_player.playing_animations().next() else {
            continue;
        };

        if anim.is_finished() {
            // info!("Transition to Idle animation");
            transitions
                .play(
                    &mut animation_player,
                    animations.idle_anim,
                    Duration::from_millis(50),
                )
                .set_speed(1.0);
        }
    }
}
