use super::{
    data::{BackgroundAssets, LoadingProgressEvent, UiAssets},
    systems::get_loading_progress_system,
};
use crate::states::AppState;
use bevy::{
    app::{Plugin, Update},
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{in_state, Condition, IntoSystemConfigs},
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt, LoadingStateSet,
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use iyes_progress::ProgressPlugin;

/// Plugin for managing asset loading states in Thetawave
pub(crate) struct ThetawaveAssetsPlugin;

impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault, //embeds assets into binary
            },
            ProgressPlugin::<AppState>::new()
                .with_state_transition(AppState::MainMenuLoading, AppState::MainMenu)
                .with_state_transition(AppState::GameLoading, AppState::Game),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_event::<LoadingProgressEvent>()
        .add_loading_state(
            LoadingState::new(AppState::MainMenuLoading)
                .load_collection::<UiAssets>()
                .load_collection::<BackgroundAssets>(),
        )
        .add_loading_state(
            LoadingState::new(AppState::GameLoading).continue_to_state(AppState::Game),
        )
        .add_systems(
            Update,
            get_loading_progress_system
                .run_if(in_state(AppState::MainMenuLoading).or(in_state(AppState::GameLoading)))
                .after(LoadingStateSet(AppState::MainMenuLoading))
                .after(LoadingStateSet(AppState::GameLoading)),
        );
    }
}
