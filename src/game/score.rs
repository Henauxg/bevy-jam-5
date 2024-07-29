use std::time::Duration;

use crate::screen::Screen;
use crate::ui::prelude::*;
use bevy::{
    app::{App, Update},
    color::{
        palettes::css::{BLUE, GREEN, RED},
        Alpha, Color,
    },
    math::Vec3,
    prelude::{
        in_state, BuildChildren, Commands, Component, DespawnRecursiveExt, Entity, Event,
        IntoSystemConfigs, OnEnter, ParamSet, Query, Res, ResMut, Resource, StateScoped, Transform,
        Trigger, With,
    },
    reflect::Reflect,
    text::{Text, TextSection, TextStyle},
    time::{Time, Timer, TimerMode},
};
use bevy_mod_billboard::BillboardTextBundle;
use bevy_tweening::{
    lens::{TextColorLens, TransformPositionLens},
    Animator, EaseFunction, Tween,
};

use super::assets::{FontKey, HandleMap, DEFAULT_FONT_KEY};

pub const DEFAULT_BAD_ACTION_SCORE: f32 = -5.;
pub const DEFAULT_GOOD_ACTION_SCORE: f32 = 10.;
pub const DEFAULT_PERFECT_ACTION_SCORE: f32 = 15.;

pub const INITIAL_DIFFICULTY_FACTOR: f32 = 1.;
pub const MAX_DIFFICULTY_FACTOR: f32 = 2.;
pub const DIFFICULTY_FACTOR_PER_SEC: f32 = 0.01;

pub const SCORE_BILLBOARDS_TEXT_DURATION_MS: u64 = 1850;
pub const SCORE_BILLBOARD_TEXT_COLOR_BAD: Color = Color::Srgba(RED);
pub const SCORE_BILLBOARD_TEXT_COLOR_GOOD: Color = Color::Srgba(BLUE);
pub const SCORE_BILLBOARD_TEXT_COLOR_PERFECT: Color = Color::Srgba(GREEN);
pub const SCORE_BILLBOARDS_TEXT_SIZE: f32 = 60.0;
pub const SCORE_BILLBOARDS_SCALE: f32 = 0.03;
pub const SCORE_BILLBOARDS_FROM_DELTA: f32 = 4.;
pub const SCORE_BILLBOARDS_TO_DELTA: f32 = 8.;

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

    app.add_systems(
        Update,
        (
            update_difficulty.run_if(in_state(Screen::Playing)),
            despawn_score_billboards,
        ),
    );

    app.observe(handle_score_actions);
    app.observe(update_score_ui);
}

#[derive(Reflect, PartialEq, Eq, Clone)]
pub enum ScoreActionType {
    Bad,
    Good,
    Perfect,
}
#[derive(Event, Clone, Reflect)]
pub struct ScoreAction {
    pub action: ScoreActionType,
    // Where it happened
    pub pos: Vec3,
}

pub fn handle_score_actions(
    trigger: Trigger<ScoreAction>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    difficulty: ResMut<Difficulty>,
    font_handles: Res<HandleMap<FontKey>>,
) {
    let score_action = &trigger.event();
    let (score_action_raw_value, billboard_text_color) = match score_action.action {
        ScoreActionType::Bad => (DEFAULT_BAD_ACTION_SCORE, SCORE_BILLBOARD_TEXT_COLOR_BAD),
        ScoreActionType::Good => (DEFAULT_GOOD_ACTION_SCORE, SCORE_BILLBOARD_TEXT_COLOR_GOOD),
        ScoreActionType::Perfect => (
            DEFAULT_PERFECT_ACTION_SCORE,
            SCORE_BILLBOARD_TEXT_COLOR_PERFECT,
        ),
    };
    let difficulty_factor = MAX_DIFFICULTY_FACTOR.min(
        INITIAL_DIFFICULTY_FACTOR + difficulty.time_elapsed_s as f32 * DIFFICULTY_FACTOR_PER_SEC,
    );
    let (rounded_action_value, action_text) = if score_action_raw_value > 0. {
        let value = (score_action_raw_value / difficulty_factor) as i32;
        (value, format!("+{}", value))
    } else {
        let value = (score_action_raw_value * difficulty_factor) as i32;
        (value, format!("{}", value))
    };

    score.current += rounded_action_value;
    if score.current > 0 && score.current as u32 > score.highscore {
        score.highscore = score.current as u32;
    }

    let Some(font) = font_handles.get(&DEFAULT_FONT_KEY) else {
        return;
    };

    // TODO Color from ActionType
    let translate_up = Tween::new(
        EaseFunction::QuarticOut,
        Duration::from_millis(SCORE_BILLBOARDS_TEXT_DURATION_MS),
        TransformPositionLens {
            start: score_action.pos + SCORE_BILLBOARDS_FROM_DELTA * Vec3::Y,
            end: score_action.pos + SCORE_BILLBOARDS_TO_DELTA * Vec3::Y,
        },
    );
    let fade_color = Tween::new(
        EaseFunction::ExponentialIn,
        Duration::from_millis(SCORE_BILLBOARDS_TEXT_DURATION_MS),
        TextColorLens {
            start: billboard_text_color,
            end: billboard_text_color.with_alpha(0.),
            section: 0,
        },
    );
    commands.spawn((
        StateScoped(Screen::Playing),
        BillboardTextBundle {
            transform: Transform::from_translation(
                score_action.pos + SCORE_BILLBOARDS_FROM_DELTA * Vec3::Y,
            )
            .with_scale(Vec3::splat(SCORE_BILLBOARDS_SCALE)),
            text: Text::from_sections([TextSection {
                value: action_text,
                style: TextStyle {
                    font_size: SCORE_BILLBOARDS_TEXT_SIZE,
                    font: font.clone_weak(),
                    color: billboard_text_color,
                    ..Default::default()
                },
            }]),
            // .with_alignment(TextAlignment::CENTER),
            ..Default::default()
        },
        // Add an Animator component to control and execute the animation.
        Animator::new(translate_up),
        Animator::new(fade_color),
        ScoreBillboard {
            timer: Timer::new(
                Duration::from_millis(SCORE_BILLBOARDS_TEXT_DURATION_MS),
                TimerMode::Once,
            ),
        },
    ));

    commands.trigger(ScoreUpdate);
    // TODO Difficulty/time
}

#[derive(Component)]
pub struct ScoreBillboard {
    timer: Timer,
}

fn despawn_score_billboards(
    mut commands: Commands,
    time: Res<Time>,
    mut score_billboards: Query<(Entity, &mut ScoreBillboard)>,
) {
    for (entity, mut billboard) in score_billboards.iter_mut() {
        billboard.timer.tick(time.delta());
        if billboard.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
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

pub fn setup_score_ui(mut commands: Commands, font_handles: Res<HandleMap<FontKey>>) {
    let font = font_handles.get(&FontKey::RomanSD).unwrap().clone();
    commands
        .bottom_ui_root()
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children.dynamic_label_with_marker("Score: ", "0", ScoreText, font.clone_weak());
        })
        .with_children(|children| {
            children.dynamic_label_with_marker(
                "Highscore: ",
                "0",
                HighscoreText,
                font.clone_weak(),
            );
        })
        .with_children(|children| {
            children.dynamic_label_with_marker(
                "Time: ",
                "0",
                DifficultyTimerText,
                font.clone_weak(),
            );
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
