//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{
        assets::{
            all_assets_loaded, all_assets_processed, process_dummy_asset, process_jug_asset,
            process_shield_asset, AssetsProcessing,
        },
        spawn::arena::SpawnArena,
    },
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AssetsProcessing>();
    app.add_systems(OnEnter(Screen::Loading), enter_loading);
    app.add_systems(
        Update,
        (
            (process_jug_asset, process_dummy_asset, process_shield_asset)
                .run_if(in_state(Screen::Loading)),
            continue_to_title.run_if(
                in_state(Screen::Loading)
                    .and_then(all_assets_processed)
                    .and_then(all_assets_loaded),
            ),
        ),
    );
}

fn enter_loading(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Loading))
        .with_children(|children| {
            children.label("Loading...");
        });
}

fn continue_to_title(mut next_screen: ResMut<NextState<Screen>>, mut commands: Commands) {
    next_screen.set(Screen::MainMenu);
    commands.trigger(SpawnArena);
}
