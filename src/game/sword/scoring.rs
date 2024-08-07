use bevy::{
    app::App,
    prelude::{Commands, Query, Trigger},
};

use crate::game::{
    score::{ScoreAction, ScoreActionType},
    spawn::dummy::Dummy,
};

use super::slicing::SliceEvent;

pub(super) fn plugin(app: &mut App) {
    app.observe(update_score);
}

pub fn update_score(
    trigger: Trigger<SliceEvent>,
    mut commands: Commands,
    // mut dummies: ResMut<Dummies>,
    dummies_query: Query<&Dummy>,
) {
    let slice_info = trigger.event();

    if let Ok(_dummy) = dummies_query.get(slice_info.entity) {
        commands.trigger(ScoreAction {
            action: ScoreActionType::Good,
            pos: slice_info.pos,
        });

        //TODO != ScoreActionType based on the dummy data
    }
}
