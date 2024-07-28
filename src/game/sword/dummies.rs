use std::time::Duration;

use bevy::{
    app::{App, Update},
    color::palettes::css::{GREEN, RED},
    ecs::component::StorageType,
    math::Vec3,
    prelude::{
        in_state, Children, Commands, Component, Entity, Event, Gizmos, IntoSystemConfigs, OnEnter,
        OnExit, Query, Res, ResMut, Resource, StateScoped, Transform, Trigger, With, Without,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
    utils::default,
};
use rand::Rng;

use crate::game::{
    arena::ArenaMode,
    spawn::{
        dummy::{Dummy, SpawnDummy},
        sword::SpawnSword,
    },
};

use super::slicing::SliceEvent;

pub const DUMMY_POSITIONS: [Vec3; 6] = [
    Vec3::new(3.5, 0., 0.5),
    Vec3::new(2.25, 0., 1.5),
    Vec3::new(1., 0., 2.25),
    Vec3::new(-1., 0., 2.25),
    Vec3::new(-2.25, 0., 1.5),
    Vec3::new(-3.5, 0., 0.5),
];

pub const DUMMY_SLOT_FREE_AFTER_SLICE_DURATION_MS: u64 = 3000;
pub const DUMMIES_SPAWN_TIMER_MS: u64 = 1200;
pub const MAX_DUMMIES_COUNT: usize = 3;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DummiesData>();

    app.add_systems(OnEnter(ArenaMode::Sword), on_enter_sword_mode);
    app.add_systems(OnExit(ArenaMode::Sword), on_exit_sword_mode);
    app.add_systems(
        Update,
        (free_dummy_slots, spawn_dummies).run_if(in_state(ArenaMode::Sword)),
    );

    app.observe(spawn_dummy_slots);
    app.observe(queue_dummy_slot_free);
}

#[derive(Resource, Reflect)]
pub struct DummiesData {
    dummy_slots: Vec<Entity>,
    free_slot_indexes: Vec<usize>,
    slot_indexes_to_free: Vec<(usize, Timer)>,

    spawn_timer: Timer,
    max_dummy_count: usize,
}

impl Default for DummiesData {
    fn default() -> Self {
        Self {
            spawn_timer: Timer::new(
                Duration::from_millis(DUMMIES_SPAWN_TIMER_MS),
                TimerMode::Once,
            ),
            max_dummy_count: MAX_DUMMIES_COUNT,
            dummy_slots: default(),
            free_slot_indexes: default(),
            slot_indexes_to_free: default(),
        }
    }
}

#[derive(Reflect)]
pub struct DummySlot;
impl Component for DummySlot {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _comp_id| {
            let mut dummies = world.resource_mut::<DummiesData>();
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

pub fn on_enter_sword_mode(mut commands: Commands) {
    commands.insert_resource({
        DummiesData {
            spawn_timer: Timer::new(
                Duration::from_millis(DUMMIES_SPAWN_TIMER_MS),
                TimerMode::Once,
            ),
            max_dummy_count: MAX_DUMMIES_COUNT,
            ..Default::default()
        }
    });
    commands.trigger(SpawnSword);
    commands.trigger(SpawnDummySlots);
}

pub fn on_exit_sword_mode(mut commands: Commands) {
    commands.remove_resource::<DummiesData>();
}

#[derive(Event, Debug)]
pub struct SpawnDummySlots;

pub fn spawn_dummy_slots(_trigger: Trigger<SpawnDummySlots>, mut commands: Commands) {
    for pos in DUMMY_POSITIONS.iter() {
        commands.spawn((
            StateScoped(ArenaMode::Sword),
            DummySlot,
            Transform::from_translation(*pos),
        ));
    }
}

pub fn queue_dummy_slot_free(
    trigger: Trigger<SliceEvent>,
    mut dummies: ResMut<DummiesData>,
    dummies_query: Query<&Dummy>,
) {
    let slice_info = trigger.event();

    if let Ok(dummy) = dummies_query.get(slice_info.entity) {
        dummies.slot_indexes_to_free.push((
            dummy.0,
            Timer::new(
                Duration::from_millis(DUMMY_SLOT_FREE_AFTER_SLICE_DURATION_MS),
                TimerMode::Once,
            ),
        ));
    }
}

pub fn free_dummy_slots(time: Res<Time>, mut dummies: ResMut<DummiesData>) {
    let mut indexes_to_remove = Vec::new();
    for (index, (_slot, timer)) in dummies.slot_indexes_to_free.iter_mut().enumerate() {
        timer.tick(time.delta());
        if timer.finished() {
            indexes_to_remove.push(index);
        }
    }
    for i in indexes_to_remove.iter() {
        let slot = dummies.slot_indexes_to_free.remove(*i);
        dummies.free_slot_indexes.push(slot.0);
    }
}

pub fn spawn_dummies(
    mut commands: Commands,
    time: Res<Time>,
    mut dummies: ResMut<DummiesData>,
    dummy_slots_query: Query<&Transform, (With<DummySlot>, Without<Children>)>,
) {
    dummies.spawn_timer.tick(time.delta());

    let occupied_dummy_slots_count = dummies.dummy_slots.len() - dummies.free_slot_indexes.len();
    if dummies.spawn_timer.finished()
        && dummies.free_slot_indexes.len() > 0
        && occupied_dummy_slots_count < dummies.max_dummy_count
    {
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..dummies.free_slot_indexes.len());
        let free_slot_index = dummies.free_slot_indexes[random_index];
        let Ok(slot) = dummy_slots_query.get(dummies.dummy_slots[free_slot_index]) else {
            return;
        };
        commands.trigger(SpawnDummy {
            pos: slot.translation,
            slot_index: free_slot_index,
        });
        dummies.spawn_timer.reset();
        dummies.free_slot_indexes.swap_remove(random_index);
        // info!("Dummy slot {} is occupied", free_slot_index);
    }
}

pub fn debug_draw_dummy_slots(
    mut gizmos: Gizmos,
    dummies: Option<Res<DummiesData>>,
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
