use std::time::Duration;

use bevy::{
    app::{App, Update},
    color::palettes::css::RED,
    ecs::component::StorageType,
    math::Vec3,
    prelude::{
        in_state, Children, Commands, Component, Condition, Entity, Event, Gizmos,
        IntoSystemConfigs, Query, Res, ResMut, Resource, Transform, Trigger, With, Without,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
};
use rand::Rng;

use crate::{
    game::{
        arena::{arena_is_in_dummies_mode, ArenaMode},
        spawn::dummy::{Dummy, SpawnDummy},
    },
    screen::Screen,
};

use super::slicing::SliceEvent;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Dummies>();
    app.register_type::<Dummies>();

    app.add_systems(
        Update,
        spawn_dummies.run_if(in_state(Screen::Playing).and_then(arena_is_in_dummies_mode)),
    );

    app.observe(spawn_dummy_slots);
    app.observe(free_dummy_slot);
    app.observe(setup_dummies_mode_data);
}

#[derive(Resource, Default, Reflect)]
pub struct Dummies {
    dummy_slots: Vec<Entity>,
    free_slot_indexes: Vec<usize>,
    spawn_timer: Timer,
    max_dummy_count: usize,
}

#[derive(Reflect)]
pub struct DummySlot;
impl Component for DummySlot {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _comp_id| {
            let mut dummies = world.resource_mut::<Dummies>();
            let slot_count = dummies.dummy_slots.len();
            dummies.free_slot_indexes.push(slot_count);
            dummies.dummy_slots.push(entity);
        });
        hooks.on_remove(|mut world, entity, _comp_id| {
            let slots = &world.resource_mut::<Dummies>().dummy_slots;
            let Some(slot_index) = slots.iter().position(|slot| *slot == entity) else {
                return;
            };
            world
                .resource_mut::<Dummies>()
                .dummy_slots
                .swap_remove(slot_index);
        });
    }
}

pub const DUMMY_POSITIONS: [Vec3; 5] = [
    Vec3::new(3., 0., 0.),
    Vec3::new(1.5, 0., 0.75),
    Vec3::new(0., 0., 1.5),
    Vec3::new(-1.5, 0., 0.75),
    Vec3::new(-3., 0., 0.),
];

#[derive(Event, Debug)]
pub struct SpawnDummySlots;

pub fn spawn_dummy_slots(_trigger: Trigger<SpawnDummySlots>, mut commands: Commands) {
    for pos in DUMMY_POSITIONS.iter() {
        commands.spawn((DummySlot, Transform::from_translation(*pos)));
    }
}

#[derive(Event, Debug)]
pub struct StartDummiesMinigame;

pub fn setup_dummies_mode_data(
    _trigger: Trigger<StartDummiesMinigame>,
    mut dummies: ResMut<Dummies>,
    mut arena_mode: ResMut<ArenaMode>,
) {
    dummies.spawn_timer = Timer::new(Duration::from_millis(1500), TimerMode::Once);
    dummies.max_dummy_count = 3;
    *arena_mode = ArenaMode::Dummies;
}

pub fn free_dummy_slot(
    trigger: Trigger<SliceEvent>,
    mut dummies: ResMut<Dummies>,
    dummies_query: Query<&Dummy>,
) {
    let slice_info = trigger.event();

    if let Ok(dummy) = dummies_query.get(slice_info.entity) {
        dummies.free_slot_indexes.push(dummy.0);
        // info!("Dummy slot {} is free", dummy.0);
    }
}

pub fn spawn_dummies(
    mut commands: Commands,
    time: Res<Time>,
    mut dummies: ResMut<Dummies>,
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

pub fn debug_draw_dummy_slots(mut gizmos: Gizmos, dummy_slots: Query<(&Transform, &DummySlot)>) {
    // TODO Draw as green for free slots
    for (slot_transform, _slot) in dummy_slots.iter() {
        gizmos.sphere(slot_transform.translation, slot_transform.rotation, 1., RED);
    }
}
