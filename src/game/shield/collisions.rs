use bevy::{
    app::{App, Update},
    prelude::{
        in_state, Commands, EventReader, IntoSystemConfigs, Query, Transform, With, Without,
    },
};
use bevy_rapier3d::prelude::CollisionEvent;

use crate::game::{
    arena::ArenaMode,
    score::{ScoreAction, ScoreActionType},
    shattering::ShatterEntity,
    spawn::shield::Shield,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, display_events.run_if(in_state(ArenaMode::Shield)));
}

fn display_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    // mut contact_force_events: EventReader<ContactForceEvent>,
    shield_query: Query<&Transform, With<Shield>>,
    other_transforms_query: Query<&Transform, Without<Shield>>,
) {
    for collision_event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _flags) = collision_event else {
            continue;
        };
        let (shield_transform, jug_entity) = if let Ok(transform) = shield_query.get(*e1) {
            (transform, *e2)
        } else if let Ok(transform) = shield_query.get(*e2) {
            (transform, *e1)
        } else {
            // TODO Jug versus ground/character ? just bounce ? At least despawn timer (could be on the jug itself already as sonn as it is spawned)
            continue;
        };
        let Ok(jug_transfrom) = other_transforms_query.get(jug_entity) else {
            continue;
        };
        // TODO action type
        commands.trigger(ScoreAction {
            action: ScoreActionType::Good,
            pos: shield_transform.translation,
        });

        let impact_direction =
            (shield_transform.translation - jug_transfrom.translation).normalize();
        // TODO Depend on speed of the jug
        let impulse = 15000.0 * impact_direction;
        commands.trigger(ShatterEntity {
            entity: jug_entity,
            impulse,
        });
        // TODO Collision with gladiator: should trigger a MissEvent (and impulse the jug in the other direction ?)
    }
}
