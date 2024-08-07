use std::time::Duration;

use crate::screen::Screen;
use crate::ui::prelude::*;
use bevy::{
    app::{App, Update},
    color::{
        palettes::css::{LIGHT_BLUE, LIGHT_GREEN, RED},
        Alpha, Color,
    },
    math::Vec3,
    prelude::{
        in_state, BuildChildren, Camera, Commands, Component, DespawnRecursiveExt, Entity, Event,
        IntoSystemConfigs, NextState, OnEnter, ParamSet, Query, Res, ResMut, Resource, StateScoped,
        Transform, Trigger, With,
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

use super::{
    arena::ArenaMode,
    assets::{FontKey, HandleMap, DEFAULT_FONT_KEY},
    cycle::Cycle,
};

pub const DEFAULT_BAD_ACTION_SCORE: f32 = -10.;
pub const DEFAULT_GOOD_ACTION_SCORE: f32 = 10.;
pub const DEFAULT_PERFECT_ACTION_SCORE: f32 = 15.;

pub const INITIAL_DIFFICULTY_FACTOR: f32 = 1.;
pub const MAX_DIFFICULTY_FACTOR: f32 = 2.;
pub const DIFFICULTY_FACTOR_PER_SEC: f32 = 0.01;

pub const SCORE_BILLBOARDS_TEXT_DURATION_MS: u64 = 1850;
pub const SCORE_BILLBOARD_TEXT_COLOR_BAD: Color = Color::Srgba(RED);
pub const SCORE_BILLBOARD_TEXT_COLOR_GOOD: Color = Color::Srgba(LIGHT_BLUE);
pub const SCORE_BILLBOARD_TEXT_COLOR_PERFECT: Color = Color::Srgba(LIGHT_GREEN);
pub const SCORE_BILLBOARDS_TEXT_SIZE: f32 = 66.0;
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
impl Difficulty {
    /// From INITIAL_DIFFICULTY_FACTOR to MAX_DIFFICULTY_FACTOR, increasing by DIFFICULTY_FACTOR_PER_SEC
    pub fn difficulty_factor(&self) -> f32 {
        MAX_DIFFICULTY_FACTOR
            .min(INITIAL_DIFFICULTY_FACTOR + self.time_elapsed_s as f32 * DIFFICULTY_FACTOR_PER_SEC)
    }

    /// difficulty_factor but from 0 to 1
    pub fn difficulty_factor_0_1(&self) -> f32 {
        self.difficulty_factor() - 1.
    }
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
    app.observe(detect_game_over);
}

#[derive(Reflect, PartialEq, Eq, Clone)]
pub enum ScoreActionType {
    Bad,
    Good,
    Perfect,
}
impl ScoreActionType {
    fn to_properties(&self) -> (f32, Color, &str) {
        match self {
            ScoreActionType::Bad => (
                DEFAULT_BAD_ACTION_SCORE,
                SCORE_BILLBOARD_TEXT_COLOR_BAD,
                "Miss",
            ),
            ScoreActionType::Good => (
                DEFAULT_GOOD_ACTION_SCORE,
                SCORE_BILLBOARD_TEXT_COLOR_GOOD,
                "Good",
            ),
            ScoreActionType::Perfect => (
                DEFAULT_PERFECT_ACTION_SCORE,
                SCORE_BILLBOARD_TEXT_COLOR_PERFECT,
                "Perfect",
            ),
        }
    }
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
    cycle: Res<Cycle>,
    mut score: ResMut<Score>,
    difficulty: ResMut<Difficulty>,
    font_handles: Res<HandleMap<FontKey>>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    let Ok(cam_transform) = camera_query.get_single() else {
        return;
    };
    let score_action = trigger.event();
    let (score_action_raw_value, billboard_text_color, action_text) =
        score_action.action.to_properties();
    let difficulty_factor = difficulty.difficulty_factor();
    let (rounded_action_value, action_text) = if score_action_raw_value > 0. {
        let value = (score_action_raw_value / difficulty_factor) as i32;
        (value, format!("{} (+{})", action_text, value))
    } else {
        let value = (score_action_raw_value * difficulty_factor) as i32;
        (value, format!("{} ({})", action_text, value))
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
    // Depends on the distance to the camera
    let billboard_scale = (cam_transform.translation - score_action.pos).length() / 80.
        * Vec3::splat(SCORE_BILLBOARDS_SCALE);
    commands.spawn((
        StateScoped(cycle.current_mode),
        BillboardTextBundle {
            transform: Transform::from_scale(billboard_scale),
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
    let font = font_handles.get(&DEFAULT_FONT_KEY).unwrap().clone();
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
        });

    commands
        .top_ui_root()
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children.dynamic_label_with_marker(
                "Time: ",
                "0",
                DifficultyTimerText,
                font.clone_weak(),
            );
        });
}

pub fn detect_game_over(
    _trigger: Trigger<ScoreUpdate>,
    score: Res<Score>,
    mut cycle: ResMut<Cycle>,
    mut next_mode: ResMut<NextState<ArenaMode>>,
) {
    if score.current < 0 {
        next_mode.set(ArenaMode::GameOver);
        cycle.current_mode = ArenaMode::GameOver;
        cycle.next_mode_timer.pause();
    }
}

pub fn update_score_ui(
    _trigger: Trigger<ScoreUpdate>,
    score: Res<Score>,
    cycle: Res<Cycle>,
    mut ui_queries: ParamSet<(
        Query<&mut Text, With<ScoreText>>,
        Query<&mut Text, With<HighscoreText>>,
    )>,
) {
    // Quick & dirty
    if cycle.current_mode == ArenaMode::GameOver {
        return;
    }
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
    cycle: Res<Cycle>,
    mut difficulty: ResMut<Difficulty>,
    mut timer_text_query: Query<&mut Text, With<DifficultyTimerText>>,
) {
    // Quick & dirty
    if cycle.current_mode == ArenaMode::GameOver {
        return;
    }

    difficulty.time_elapsed_s += time.delta_seconds();

    let Ok(mut timer_text) = timer_text_query.get_single_mut() else {
        return;
    };
    timer_text.sections[1].value = (difficulty.time_elapsed_s as usize).to_string();
}
