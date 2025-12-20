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
    ExtendedBackgroundAssets, ExtendedMusicAssets,
    data::{ExtendedGameAssets, ExtendedUiAssets, LoadingProgressEvent},
};

use super::{
    data::{BackgroundAssets, GameAssets, MusicAssets, UiAssets},
    systems::{
        get_loading_progress_system, log_game_assets_system, log_main_menu_assets_system,
        setup_particle_materials_system, unload_game_assets_system,
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

        // Configure main menu loading state
        let main_menu_loading = LoadingState::new(AppState::MainMenuLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
            .load_collection::<UiAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("extended://ui.assets.ron")
            .load_collection::<ExtendedUiAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("music.assets.ron")
            .load_collection::<MusicAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "extended://music.assets.ron",
            )
            .load_collection::<ExtendedMusicAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("background.assets.ron")
            .load_collection::<BackgroundAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "extended://background.assets.ron",
            )
            .load_collection::<ExtendedBackgroundAssets>();

        // Configure game loading state
        let game_loading = LoadingState::new(AppState::GameLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game.assets.ron")
            .load_collection::<GameAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "extended://game.assets.ron",
            )
            .load_collection::<ExtendedGameAssets>();

        // ProgressPlugin handles state transitions
        app.add_loading_state(main_menu_loading)
            .add_loading_state(game_loading);

        app.add_systems(
            OnEnter(AppState::GameLoading),
            setup_particle_materials_system,
        )
        .add_systems(OnEnter(AppState::MainMenu), log_main_menu_assets_system)
        .add_systems(OnEnter(AppState::Game), log_game_assets_system)
        .add_systems(OnExit(AppState::Game), unload_game_assets_system);
    }
}
