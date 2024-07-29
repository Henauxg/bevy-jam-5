use std::time::Duration;

use bevy::{
    app::{App, Update},
    color::palettes::css::{GREEN, RED},
    ecs::component::StorageType,
    math::Vec3,
    prelude::{
        in_state, Children, Commands, Component, DespawnRecursiveExt, Entity, Event, Gizmos,
        IntoSystemConfigs, Query, Res, ResMut, Resource, StateScoped, Transform, Trigger, With,
        Without,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
    utils::default,
};
use rand::Rng;

use crate::game::{
    arena::ArenaMode,
    cycle::Cycle,
    score::{ScoreAction, ScoreActionType},
    spawn::dummy::{Dummy, SpawnDummy},
};

use super::slicing::SliceEvent;

pub const DUMMY_POSITIONS: [Vec3; 6] = [
    Vec3::new(4., 0., 1.),
    Vec3::new(2.75, 0., 2.5),
    Vec3::new(1.5, 0., 3.),
    Vec3::new(-1.5, 0., 3.),
    Vec3::new(-2.75, 0., 2.5),
    Vec3::new(-4., 0., 1.),
];

pub const DUMMY_SLOT_FREE_AFTER_SLICE_DURATION_MS: u64 = 3000;
// pub const DUMMIES_SPAWN_TIMER_MS: u64 = 1200;
pub const DUMMIES_SPAWN_INTERVAL_MIN_MS: u64 = 550;
pub const DUMMIES_SPAWN_INTERVAL_MAX_MS: u64 = 1450;
pub const MAX_DUMMIES_COUNT: usize = 4;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DummiesModeData>();

    // app.add_systems(OnEnter(ArenaMode::Sword), on_enter_sword_mode);
    // app.add_systems(OnExit(ArenaMode::Sword), on_exit_sword_mode);
    app.add_systems(
        Update,
        (free_killed_dummies_slots, spawn_dummies, despawn_dummies)
            .run_if(in_state(ArenaMode::Sword)),
    );

    app.observe(spawn_dummy_slots);
    app.observe(queue_dummy_slot_free);
}

#[derive(Resource, Reflect)]
pub struct DummiesModeData {
    dummy_slots: Vec<Entity>,
    free_slot_indexes: Vec<usize>,
    killed_dummies_queue: Vec<(usize, Timer)>,

    spawn_timer: Timer,
    max_dummy_count: usize,
}

impl Default for DummiesModeData {
    fn default() -> Self {
        Self {
            spawn_timer: Timer::new(
                Duration::from_millis(DUMMIES_SPAWN_INTERVAL_MIN_MS),
                TimerMode::Once,
            ),
            max_dummy_count: MAX_DUMMIES_COUNT,
            dummy_slots: default(),
            free_slot_indexes: default(),
            killed_dummies_queue: default(),
        }
    }
}

#[derive(Reflect)]
pub struct DummySlot;
impl Component for DummySlot {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _comp_id| {
            let mut dummies = world.resource_mut::<DummiesModeData>();
            let slot_count = dummies.dummy_slots.len();
            dummies.free_slot_indexes.push(slot_count);
            dummies.dummy_slots.push(entity);
        });
        // TODO Not needed, Would also need to update free_slot_indexes and slot_indexes_to_free
        // hooks.on_remove(|mut world, entity, _comp_id| {
        //     let slots = &world.resource_mut::<DummiesData>().dummy_slots;
        //     let Some(slot_index) = slots.iter().position(|slot| *slot == entity) else {
        //         return;
        //     };
        //     world
        //         .resource_mut::<DummiesData>()
        //         .dummy_slots
        //         .swap_remove(slot_index);
        // });
    }
}

#[derive(Event, Debug)]
pub struct SpawnDummySlots;

fn spawn_dummy_slots(_trigger: Trigger<SpawnDummySlots>, mut commands: Commands) {
    for pos in DUMMY_POSITIONS.iter() {
        commands.spawn((
            StateScoped(ArenaMode::Sword),
            DummySlot,
            Transform::from_translation(*pos),
        ));
    }
}

fn queue_dummy_slot_free(
    trigger: Trigger<SliceEvent>,
    mut dummies: ResMut<DummiesModeData>,
    dummies_query: Query<&Dummy>,
) {
    let slice_info = trigger.event();

    if let Ok(dummy) = dummies_query.get(slice_info.entity) {
        dummies.killed_dummies_queue.push((
            dummy.slot_index,
            Timer::new(
                Duration::from_millis(DUMMY_SLOT_FREE_AFTER_SLICE_DURATION_MS),
                TimerMode::Once,
            ),
        ));
    }
}

fn free_killed_dummies_slots(time: Res<Time>, mut dummies: ResMut<DummiesModeData>) {
    let mut indexes_to_remove = Vec::new();
    for (index, (_slot, timer)) in dummies.killed_dummies_queue.iter_mut().enumerate() {
        timer.tick(time.delta());
        if timer.finished() {
            indexes_to_remove.push(index);
        }
    }
    for i in indexes_to_remove.iter() {
        let slot = dummies.killed_dummies_queue.remove(*i);
        dummies.free_slot_indexes.push(slot.0);
    }
}

fn spawn_dummies(
    mut commands: Commands,
    time: Res<Time>,
    cycle: Res<Cycle>,
    mut dummies_mode: ResMut<DummiesModeData>,
    dummy_slots_query: Query<&Transform, (With<DummySlot>, Without<Children>)>,
) {
    dummies_mode.spawn_timer.tick(time.delta());

    let occupied_dummy_slots_count =
        dummies_mode.dummy_slots.len() - dummies_mode.free_slot_indexes.len();
    if dummies_mode.spawn_timer.finished()
        && dummies_mode.free_slot_indexes.len() > 0
        && occupied_dummy_slots_count < dummies_mode.max_dummy_count
    {
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..dummies_mode.free_slot_indexes.len());
        let free_slot_index = dummies_mode.free_slot_indexes[random_index];
        let Ok(slot) = dummy_slots_query.get(dummies_mode.dummy_slots[free_slot_index]) else {
            return;
        };
        commands.trigger(SpawnDummy {
            pos: slot.translation,
            slot_index: free_slot_index,
            scope: cycle.current_mode,
        });
        let mut rng = rand::thread_rng();
        let next_spawn_delay =
            rng.gen_range(DUMMIES_SPAWN_INTERVAL_MIN_MS..DUMMIES_SPAWN_INTERVAL_MAX_MS);
        dummies_mode
            .spawn_timer
            .set_duration(Duration::from_millis(next_spawn_delay));
        dummies_mode.spawn_timer.reset();
        dummies_mode.free_slot_indexes.swap_remove(random_index);
    }
}

fn despawn_dummies(
    mut commands: Commands,
    time: Res<Time>,
    mut dummies_mode_data: ResMut<DummiesModeData>,
    mut dummies_query: Query<(Entity, &Transform, &mut Dummy)>,
) {
    for (entity, transform, mut dummy) in dummies_query.iter_mut() {
        dummy.despawn_timer.tick(time.delta());
        if dummy.despawn_timer.finished() {
            // TODO Despawn this dummy with an animation
            commands.entity(entity).despawn_recursive();
            commands.trigger(ScoreAction {
                action: ScoreActionType::Bad,
                pos: transform.translation,
            });
            dummies_mode_data.free_slot_indexes.push(dummy.slot_index);
        }
    }
}

pub fn debug_draw_dummy_slots(
    mut gizmos: Gizmos,
    dummies: Option<Res<DummiesModeData>>,
    dummy_slots: Query<&Transform, With<DummySlot>>,
) {
    let Some(dummies_data) = dummies else {
        return;
    };
    for (slot_index, &slot_entity) in dummies_data.dummy_slots.iter().enumerate() {
        let Ok(slot_transform) = dummy_slots.get(slot_entity) else {
            continue;
        };
        let color = if dummies_data.free_slot_indexes.contains(&slot_index) {
            GREEN
        } else {
            RED
        };
        gizmos.sphere(
            slot_transform.translation,
            slot_transform.rotation,
            1.,
            color,
        );
    }
}
