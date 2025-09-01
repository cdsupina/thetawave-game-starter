use crate::data::ExtendedGameAssets;

use super::{
    data::{BackgroundAssets, GameAssets, LoadingProgressEvent, MusicAssets, UiAssets},
    systems::{
        get_loading_progress_system, setup_particle_materials_system, unload_game_assets_system,
    },
};
use bevy::{
    app::{Plugin, Update},
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{Condition, IntoScheduleConfigs, in_state},
    state::state::{OnEnter, OnExit},
};
use bevy_asset_loader::{
    loading_state::{
        LoadingState, LoadingStateAppExt, LoadingStateSet, config::ConfigureLoadingState,
    },
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
use iyes_progress::ProgressPlugin;
use thetawave_states::AppState;

/// Plugin for managing asset loading states in Thetawave
#[derive(Default)]
pub struct ThetawaveAssetsPlugin;

impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            ProgressPlugin::<AppState>::new()
                .with_state_transition(AppState::MainMenuLoading, AppState::MainMenu)
                .with_state_transition(AppState::GameLoading, AppState::Game),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_event::<LoadingProgressEvent>()
        .add_loading_state(
            LoadingState::new(AppState::MainMenuLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
                .load_collection::<UiAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("music.assets.ron")
                .load_collection::<MusicAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("background.assets.ron")
                .load_collection::<BackgroundAssets>(),
        )
        .add_loading_state(
            LoadingState::new(AppState::GameLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game.assets.ron")
                .load_collection::<GameAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "extended/game.assets.ron",
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
