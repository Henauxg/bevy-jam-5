use crate::screen::Screen;
use crate::ui::prelude::*;
use bevy::{
    app::{App, Update},
    prelude::{
        in_state, BuildChildren, Commands, Component, Event, IntoSystemConfigs, OnEnter, ParamSet,
        Query, Res, ResMut, Resource, StateScoped, Trigger, With,
    },
    reflect::Reflect,
    text::Text,
    time::Time,
};

pub const DEFAULT_BAD_ACTION_SCORE: f32 = -5.;
pub const DEFAULT_GOOD_ACTION_SCORE: f32 = 10.;
pub const DEFAULT_PERFECT_ACTION_SCORE: f32 = 15.;

pub const INITIAL_DIFFICULTY_FACTOR: f32 = 1.;
pub const MAX_DIFFICULTY_FACTOR: f32 = 2.;
pub const DIFFICULTY_FACTOR_PER_SEC: f32 = 0.01;

#[derive(Resource, Reflect, Clone)]
pub struct Score {
    highscore: u32,
    current: i32,
}

#[derive(Resource, Reflect, Clone)]
pub struct Difficulty {
    time_elapsed_s: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Score>();
    app.register_type::<Difficulty>();

    app.add_systems(OnEnter(Screen::Playing), (setup_score_ui, setup_score));

    app.add_systems(Update, update_difficulty.run_if(in_state(Screen::Playing)));

    app.observe(update_score);
    app.observe(update_score_ui);
}

#[derive(Reflect, PartialEq, Eq, Clone)]
pub enum ScoreActionType {
    Bad,
    Good,
    Perfect,
}
#[derive(Event, Clone, Reflect)]
pub struct ScoreAction(pub ScoreActionType);

pub fn update_score(
    trigger: Trigger<ScoreAction>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    difficulty: ResMut<Difficulty>,
) {
    let score_action_value = match trigger.event().0 {
        ScoreActionType::Bad => DEFAULT_BAD_ACTION_SCORE,
        ScoreActionType::Good => DEFAULT_GOOD_ACTION_SCORE,
        ScoreActionType::Perfect => DEFAULT_PERFECT_ACTION_SCORE,
    };
    let difficulty_factor = MAX_DIFFICULTY_FACTOR.min(
        INITIAL_DIFFICULTY_FACTOR + difficulty.time_elapsed_s as f32 * DIFFICULTY_FACTOR_PER_SEC,
    );
    let score_value = if score_action_value > 0. {
        score_action_value / difficulty_factor
    } else {
        score_action_value * difficulty_factor
    };

    score.current += score_value as i32;
    if score.current > 0 && score.current as u32 > score.highscore {
        score.highscore = score.current as u32;
    }

    commands.trigger(ScoreUpdate);
    // TODO Difficulty/time
}

#[derive(Event, Clone, Reflect)]
pub struct ScoreUpdate;

pub fn setup_score(mut commands: Commands) {
    commands.insert_resource(Score {
        highscore: 0,
        current: 0,
    });
    commands.insert_resource(Difficulty { time_elapsed_s: 0. });
}

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HighscoreText;

#[derive(Component)]
pub struct DifficultyTimerText;

pub fn setup_score_ui(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children.dynamic_label_with_marker("Score ", "0", ScoreText);
        })
        .with_children(|children| {
            children.dynamic_label_with_marker("Highscore ", "0", HighscoreText);
        })
        .with_children(|children| {
            children.dynamic_label_with_marker("Time ", "0", DifficultyTimerText);
        });
}

pub fn update_score_ui(
    _trigger: Trigger<ScoreUpdate>,
    score: Res<Score>,
    mut ui_queries: ParamSet<(
        Query<&mut Text, With<ScoreText>>,
        Query<&mut Text, With<HighscoreText>>,
    )>,
) {
    {
        let mut score_query = ui_queries.p0();
        let Ok(mut score_text) = score_query.get_single_mut() else {
            return;
        };
        score_text.sections[1].value = score.current.to_string();
    }
    {
        let mut highscore_query = ui_queries.p1();
        let Ok(mut highscore_text) = highscore_query.get_single_mut() else {
            return;
        };
        highscore_text.sections[1].value = score.highscore.to_string();
    }
}

pub fn update_difficulty(
    time: Res<Time>,
    mut difficulty: ResMut<Difficulty>,
    mut timer_text_query: Query<&mut Text, With<DifficultyTimerText>>,
) {
    difficulty.time_elapsed_s += time.delta_seconds();

    let Ok(mut timer_text) = timer_text_query.get_single_mut() else {
        return;
    };
    timer_text.sections[1].value = (difficulty.time_elapsed_s as usize).to_string();
}
