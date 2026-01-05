//! Plugin for managing asset loading in Thetawave.
//!
//! Uses bevy_asset_loader for managed loading states with iyes_progress for loading progress UI.

use bevy::{
    app::{Plugin, Update},
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::schedule::SystemCondition,
    prelude::{IntoScheduleConfigs, in_state},
    state::state::{OnEnter, OnExit},
};
use bevy_asset_loader::{
    loading_state::{
        LoadingState, LoadingStateAppExt, LoadingStateSet, config::ConfigureLoadingState,
    },
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
use iyes_progress::ProgressPlugin;
use thetawave_core::AppState;

use crate::{
    ExtendedBackgroundAssets, ExtendedMusicAssets, ModBackgroundAssets, ModGameAssets,
    ModMusicAssets, ModUiAssets,
    data::{ExtendedGameAssets, ExtendedUiAssets, LoadingProgressEvent},
};

use super::{
    data::{BackgroundAssets, GameAssets, MusicAssets, UiAssets},
    systems::{
        get_loading_progress_system, log_game_assets_system, log_main_menu_assets_system,
        merge_game_assets_system, merge_main_menu_assets_system, setup_particle_materials_system,
        unload_game_assets_system, unload_merged_game_assets_system,
    },
};

/// Plugin for managing asset loading states in Thetawave
#[derive(Default)]
pub struct ThetawaveAssetsPlugin;

impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

        // Add progress tracking plugin for loading state transitions
        app.add_plugins(
            ProgressPlugin::<AppState>::new()
                .with_state_transition(AppState::MainMenuLoading, AppState::MainMenu)
                .with_state_transition(AppState::GameLoading, AppState::Game),
        )
        .add_message::<LoadingProgressEvent>()
        .add_systems(
            Update,
            get_loading_progress_system
                .run_if(in_state(AppState::MainMenuLoading).or(in_state(AppState::GameLoading)))
                .after(LoadingStateSet(AppState::MainMenuLoading))
                .after(LoadingStateSet(AppState::GameLoading)),
        );

        // Configure main menu loading state (3-tier: base -> game -> mods)
        let main_menu_loading = LoadingState::new(AppState::MainMenuLoading)
            // Tier 1: Base assets (embedded in library)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
            .load_collection::<UiAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("music.assets.ron")
            .load_collection::<MusicAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("background.assets.ron")
            .load_collection::<BackgroundAssets>()
            // Tier 2: Game assets (developer's assets folder)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game://ui.assets.ron")
            .load_collection::<ExtendedUiAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game://music.assets.ron")
            .load_collection::<ExtendedMusicAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "game://background.assets.ron",
            )
            .load_collection::<ExtendedBackgroundAssets>()
            // Tier 3: Mod assets (user/modder assets relative to executable)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("mods://ui.assets.ron")
            .load_collection::<ModUiAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("mods://music.assets.ron")
            .load_collection::<ModMusicAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "mods://background.assets.ron",
            )
            .load_collection::<ModBackgroundAssets>();

        // Configure game loading state (3-tier: base -> game -> mods)
        let game_loading = LoadingState::new(AppState::GameLoading)
            // Tier 1: Base assets
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game.assets.ron")
            .load_collection::<GameAssets>()
            // Tier 2: Game assets
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game://game.assets.ron")
            .load_collection::<ExtendedGameAssets>()
            // Tier 3: Mod assets
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("mods://game.assets.ron")
            .load_collection::<ModGameAssets>();

        // ProgressPlugin handles state transitions
        app.add_loading_state(main_menu_loading)
            .add_loading_state(game_loading);

        app.add_systems(
            OnEnter(AppState::GameLoading),
            setup_particle_materials_system,
        )
        // Merge assets after loading completes (before logging)
        .add_systems(
            OnEnter(AppState::MainMenu),
            (merge_main_menu_assets_system, log_main_menu_assets_system).chain(),
        )
        .add_systems(
            OnEnter(AppState::Game),
            (merge_game_assets_system, log_game_assets_system).chain(),
        )
        .add_systems(
            OnExit(AppState::Game),
            (unload_game_assets_system, unload_merged_game_assets_system),
        );
    }
}
