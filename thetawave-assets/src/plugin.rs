//! Plugin for managing asset loading in Thetawave.
//!
//! When the `asset_loader` feature is enabled, uses bevy_asset_loader and iyes_progress
//! for managed loading states with progress tracking.
//!
//! When disabled, manually loads assets and immediately transitions states.

use bevy::{
    app::Plugin,
    diagnostic::FrameTimeDiagnosticsPlugin,
    state::state::{OnEnter, OnExit},
};
use thetawave_core::AppState;

#[cfg(feature = "asset_loader")]
use bevy::app::Update;

#[cfg(feature = "asset_loader")]
use bevy::prelude::{IntoScheduleConfigs, in_state};

#[cfg(feature = "asset_loader")]
use bevy::ecs::schedule::SystemCondition;

#[cfg(not(feature = "asset_loader"))]
use bevy::prelude::IntoScheduleConfigs;

use super::systems::{setup_particle_materials_system, unload_game_assets_system};

// ============================================================================
// Feature-gated imports
// ============================================================================

#[cfg(feature = "asset_loader")]
use crate::{
    ExtendedBackgroundAssets, ExtendedMusicAssets,
    data::{ExtendedGameAssets, ExtendedUiAssets, LoadingProgressEvent},
};

#[cfg(feature = "asset_loader")]
use super::{
    data::{BackgroundAssets, GameAssets, MusicAssets, UiAssets},
    systems::get_loading_progress_system,
};

#[cfg(feature = "asset_loader")]
use bevy_asset_loader::{
    loading_state::{
        LoadingState, LoadingStateAppExt, LoadingStateSet, config::ConfigureLoadingState,
    },
    standard_dynamic_asset::StandardDynamicAssetCollection,
};

#[cfg(feature = "asset_loader")]
use iyes_progress::ProgressPlugin;

#[cfg(not(feature = "asset_loader"))]
use crate::manual_loader;

// ============================================================================
// Plugin definition
// ============================================================================

/// Plugin for managing asset loading states in Thetawave
#[derive(Default)]
pub struct ThetawaveAssetsPlugin;

// ============================================================================
// Implementation with asset_loader feature
// ============================================================================

#[cfg(feature = "asset_loader")]
impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            ProgressPlugin::<AppState>::new()
                .with_state_transition(AppState::MainMenuLoading, AppState::MainMenu)
                .with_state_transition(AppState::GameLoading, AppState::Game),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_message::<LoadingProgressEvent>()
        .add_loading_state(
            LoadingState::new(AppState::MainMenuLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
                .load_collection::<UiAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "extended://ui.assets.ron",
                )
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
                .load_collection::<ExtendedBackgroundAssets>(),
        )
        .add_loading_state(
            LoadingState::new(AppState::GameLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game.assets.ron")
                .load_collection::<GameAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "extended://game.assets.ron",
                )
                .load_collection::<ExtendedGameAssets>(),
        )
        .add_systems(
            Update,
            get_loading_progress_system
                .run_if(in_state(AppState::MainMenuLoading).or(in_state(AppState::GameLoading)))
                .after(LoadingStateSet(AppState::MainMenuLoading))
                .after(LoadingStateSet(AppState::GameLoading)),
        )
        .add_systems(
            OnEnter(AppState::GameLoading),
            setup_particle_materials_system,
        )
        .add_systems(OnExit(AppState::Game), unload_game_assets_system);
    }
}

// ============================================================================
// Implementation without asset_loader feature
// ============================================================================

#[cfg(not(feature = "asset_loader"))]
impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(
                OnEnter(AppState::MainMenuLoading),
                (
                    manual_loader::load_ui_assets_system,
                    manual_loader::load_music_assets_system,
                    manual_loader::load_background_assets_system,
                    manual_loader::transition_to_main_menu,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(AppState::GameLoading),
                (
                    manual_loader::load_game_assets_system,
                    setup_particle_materials_system,
                    manual_loader::transition_to_game,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppState::Game), unload_game_assets_system);
    }
}
