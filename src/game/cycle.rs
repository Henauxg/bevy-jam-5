use std::time::Duration;

use crate::{
    ui::widgets::{Containers, Widgets},
    AppSet,
};
use bevy::{
    app::{App, Update},
    prelude::{
        in_state, BuildChildren, Commands, Component, ImageBundle, IntoSystemConfigs, NextState,
        OnEnter, Query, Res, ResMut, Resource, StateScoped, With,
    },
    reflect::Reflect,
    text::Text,
    time::{Time, Timer, TimerMode},
    ui::{Style, UiImage, Val},
};

use crate::screen::Screen;

use super::{
    arena::ArenaMode,
    assets::{FontKey, HandleMap, ImageKey, DEFAULT_FONT_KEY},
};

pub const NEXT_WEAPON_CYCLE_INTERVAL: u64 = 13000;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Playing),
        (setup_cycle, setup_cycle_ui).chain(),
    );

    app.add_systems(
        Update,
        update_cycle
            .in_set(AppSet::TickTimers)
            .run_if(in_state(Screen::Playing)),
    );
    app.add_systems(
        Update,
        update_cycle_ui
            .in_set(AppSet::Update)
            .run_if(in_state(Screen::Playing)),
    );
}

#[derive(Resource, Debug, Reflect)]
pub struct Cycle {
    pub current_mode: ArenaMode,
    pub next_mode: ArenaMode,
    pub next_mode_timer: Timer,
}

fn setup_cycle(mut commands: Commands, mut next_arena_mode: ResMut<NextState<ArenaMode>>) {
    let random_arena_mode: ArenaMode = rand::random();
    next_arena_mode.set(random_arena_mode.clone());

    let next_mode = get_random_different_mode(&random_arena_mode);
    commands.insert_resource(Cycle {
        current_mode: random_arena_mode,
        next_mode,
        next_mode_timer: Timer::new(
            Duration::from_millis(NEXT_WEAPON_CYCLE_INTERVAL),
            TimerMode::Repeating,
        ),
    });
}

fn get_random_different_mode(mode: &ArenaMode) -> ArenaMode {
    let mut random_mode = rand::random();
    while random_mode == *mode {
        random_mode = rand::random();
    }
    random_mode
}

#[derive(Component)]
pub struct NextCycleTimerText;

#[derive(Component)]
pub struct NextCycleImage;

fn setup_cycle_ui(
    mut commands: Commands,
    cycle: Res<Cycle>,
    font_handles: Res<HandleMap<FontKey>>,
    image_handles: Res<HandleMap<ImageKey>>,
) {
    let font = font_handles.get(&DEFAULT_FONT_KEY).unwrap().clone();
    let image = image_handles.get(&ImageKey::Sword).unwrap().clone_weak();
    commands
        .bottom_left_ui_root()
        .insert(StateScoped(Screen::Playing))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    image: UiImage::new(image),
                    style: Style {
                        max_width: Val::Px(90.),
                        max_height: Val::Px(90.),
                        // min_width: Val::Px(90.),
                        // min_height: Val::Px(90.),
                        align_content: bevy::ui::AlignContent::Center,
                        justify_content: bevy::ui::JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                NextCycleImage,
            ));
        })
        .with_children(|children| {
            children.dynamic_label_with_marker(
                "Next weapon in ",
                format!("{}s", cycle.next_mode_timer.remaining().as_secs()),
                NextCycleTimerText,
                font.clone_weak(),
            );
        });
}

fn update_cycle(
    // mut commands: Commands,
    time: Res<Time>,
    mut cycle: ResMut<Cycle>,
    mut next_arena_mode: ResMut<NextState<ArenaMode>>,
) {
    cycle.next_mode_timer.tick(time.delta());
    if cycle.next_mode_timer.finished() {
        next_arena_mode.set(cycle.next_mode.clone());
        cycle.current_mode = cycle.next_mode;
        cycle.next_mode = get_random_different_mode(&cycle.next_mode);
    }
}

fn update_cycle_ui(
    cycle: Res<Cycle>,
    mut next_cycle_timer_text: Query<&mut Text, With<NextCycleTimerText>>,
    mut next_cycle_image: Query<&mut UiImage, With<NextCycleImage>>,
    image_handles: Res<HandleMap<ImageKey>>,
) {
    let Ok(mut timer_text) = next_cycle_timer_text.get_single_mut() else {
        return;
    };
    let Ok(mut cycle_image) = next_cycle_image.get_single_mut() else {
        return;
    };
    timer_text.sections[1].value = format!("{}s", cycle.next_mode_timer.remaining().as_secs());

    let Some(image_handle) = image_handles.get(&cycle.next_mode.to_image_key()) else {
        return;
    };
    cycle_image.texture = image_handle.clone_weak();
}
