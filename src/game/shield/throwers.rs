use std::time::Duration;

use bevy::{
    app::{App, Update},
    ecs::component::StorageType,
    math::Vec3,
    prelude::{
        in_state, Commands, Component, Entity, Event, IntoSystemConfigs, Res, ResMut, Resource,
        Trigger,
    },
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
};
use rand::Rng;

use crate::game::{
    arena::ArenaMode,
    spawn::{
        arena::DEFAULT_GLADIATOR_POS,
        jug_thrower::{SpawnJugThrower, ThrowJug},
    },
};

pub const JUG_SPAWN_RADIUS: f32 = 12.;
pub const JUG_THROWERS_POSITIONS: [Vec3; 8] = [
    Vec3::new(-JUG_SPAWN_RADIUS, 0., 0.),
    Vec3::new(JUG_SPAWN_RADIUS, 0., 0.),
    Vec3::new(JUG_SPAWN_RADIUS * 0.75, 0., JUG_SPAWN_RADIUS * 0.75),
    Vec3::new(-JUG_SPAWN_RADIUS * 0.75, 0., JUG_SPAWN_RADIUS * 0.75),
    Vec3::new(-JUG_SPAWN_RADIUS * 0.75, 0., -JUG_SPAWN_RADIUS * 0.75),
    Vec3::new(JUG_SPAWN_RADIUS * 0.75, 0., -JUG_SPAWN_RADIUS * 0.75),
    Vec3::new(0., 0., JUG_SPAWN_RADIUS),
    Vec3::new(0., 0., -JUG_SPAWN_RADIUS),
];

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ThrowersData>();
    app.register_type::<Thrower>();

    app.add_systems(Update, (throw_jugs).run_if(in_state(ArenaMode::Shield)));

    app.observe(spawn_throwers);
}

#[derive(Resource, Reflect)]
pub struct ThrowersData {
    throwers: Vec<Entity>,
    // TODO If throwing takes time, needs to pick a free thrower
    next_throw_timer: Timer,
    min_throw_interval_ms: u64,
    max_throw_interval_ms: u64,
}
pub const INITIAL_MIN_THROW_INTERVAL_MS: u64 = 500;
pub const INITIAL_MAX_THROW_INTERVAL_MS: u64 = 1500;

impl Default for ThrowersData {
    fn default() -> Self {
        Self {
            throwers: Default::default(),
            next_throw_timer: Timer::new(
                Duration::from_millis(INITIAL_MAX_THROW_INTERVAL_MS),
                TimerMode::Once,
            ),
            min_throw_interval_ms: INITIAL_MIN_THROW_INTERVAL_MS,
            max_throw_interval_ms: INITIAL_MAX_THROW_INTERVAL_MS,
        }
    }
}

#[derive(Event, Debug)]
pub struct SpawnJugThrowers;

fn spawn_throwers(_trigger: Trigger<SpawnJugThrowers>, mut commands: Commands) {
    for pos in JUG_THROWERS_POSITIONS.iter() {
        commands.trigger(SpawnJugThrower {
            pos: *pos,
            looking_at: DEFAULT_GLADIATOR_POS,
        });
    }
}

#[derive(Reflect)]
pub struct Thrower;

impl Component for Thrower {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _comp_id| {
            let mut data = world.resource_mut::<ThrowersData>();
            data.throwers.push(entity);
            // let slot_count = throwers.dummy_slots.len();
            // throwers.free_slot_indexes.push(slot_count);
            // throwers.dummy_slots.push(entity);
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

fn throw_jugs(mut commands: Commands, time: Res<Time>, mut jug_throwers: ResMut<ThrowersData>) {
    jug_throwers.next_throw_timer.tick(time.delta());
    if jug_throwers.next_throw_timer.finished() {
        let mut rng = rand::thread_rng();
        // TODO Throw animation/build up
        let index = rng.gen_range(0..jug_throwers.throwers.len());
        commands.trigger_targets(
            ThrowJug {
                at: DEFAULT_GLADIATOR_POS,
            },
            jug_throwers.throwers[index],
        );

        // Prepare next throw
        jug_throwers.next_throw_timer =
            Timer::new(
                Duration::from_millis(rng.gen_range(
                    jug_throwers.min_throw_interval_ms..jug_throwers.max_throw_interval_ms,
                )),
                TimerMode::Once,
            );
    }
}
