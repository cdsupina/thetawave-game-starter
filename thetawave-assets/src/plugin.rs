//! Plugin for managing asset loading in Thetawave.
//!
//! Uses bevy_asset_loader for managed loading states.
//! When `progress_tracking` feature is enabled, adds iyes_progress for loading progress UI.

use bevy::{
    app::Plugin,
    diagnostic::FrameTimeDiagnosticsPlugin,
    state::state::{OnEnter, OnExit},
};
use thetawave_core::AppState;

use bevy_asset_loader::{
    loading_state::{LoadingState, LoadingStateAppExt, config::ConfigureLoadingState},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};

#[cfg(feature = "progress_tracking")]
use bevy::app::Update;

#[cfg(feature = "progress_tracking")]
use bevy::prelude::IntoScheduleConfigs;

#[cfg(feature = "progress_tracking")]
use bevy_asset_loader::loading_state::LoadingStateSet;

use crate::{
    ExtendedBackgroundAssets, ExtendedMusicAssets,
    data::{ExtendedGameAssets, ExtendedUiAssets},
};

use super::{
    data::{BackgroundAssets, GameAssets, MusicAssets, UiAssets},
    systems::{setup_particle_materials_system, unload_game_assets_system},
};

// Progress tracking feature imports
#[cfg(feature = "progress_tracking")]
use bevy::prelude::in_state;

#[cfg(feature = "progress_tracking")]
use bevy::ecs::schedule::SystemCondition;

#[cfg(feature = "progress_tracking")]
use crate::data::LoadingProgressEvent;

#[cfg(feature = "progress_tracking")]
use super::systems::get_loading_progress_system;

#[cfg(feature = "progress_tracking")]
use iyes_progress::ProgressPlugin;

/// Plugin for managing asset loading states in Thetawave
#[derive(Default)]
pub struct ThetawaveAssetsPlugin;

impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

        // Add progress tracking plugin when feature is enabled
        #[cfg(feature = "progress_tracking")]
        {
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
        }

        // Configure main menu loading state
        let main_menu_loading = LoadingState::new(AppState::MainMenuLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
            .load_collection::<UiAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("extended://ui.assets.ron")
            .load_collection::<ExtendedUiAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("music.assets.ron")
            .load_collection::<MusicAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("extended://music.assets.ron")
            .load_collection::<ExtendedMusicAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("background.assets.ron")
            .load_collection::<BackgroundAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("extended://background.assets.ron")
            .load_collection::<ExtendedBackgroundAssets>();

        // Configure game loading state
        let game_loading = LoadingState::new(AppState::GameLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game.assets.ron")
            .load_collection::<GameAssets>()
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("extended://game.assets.ron")
            .load_collection::<ExtendedGameAssets>();

        // When progress_tracking is disabled, use built-in state transitions
        #[cfg(not(feature = "progress_tracking"))]
        {
            app.add_loading_state(main_menu_loading.continue_to_state(AppState::MainMenu))
                .add_loading_state(game_loading.continue_to_state(AppState::Game));
        }

        // When progress_tracking is enabled, ProgressPlugin handles transitions
        #[cfg(feature = "progress_tracking")]
        {
            app.add_loading_state(main_menu_loading)
                .add_loading_state(game_loading);
        }

        app.add_systems(
            OnEnter(AppState::GameLoading),
            setup_particle_materials_system,
        )
        .add_systems(OnExit(AppState::Game), unload_game_assets_system);
    }
}
