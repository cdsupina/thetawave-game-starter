use super::data::{BackgroundAssets, UiAssets};
use crate::states::AppState;
use bevy::app::Plugin;
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

/// Plugin for managing asset loading states in Thetawave
pub(crate) struct ThetawaveAssetsPlugin;

impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EmbeddedAssetPlugin {
            mode: PluginMode::ReplaceDefault, //embeds assets into binary
        })
        .add_loading_state(
            LoadingState::new(AppState::MainMenuLoading)
                .continue_to_state(AppState::MainMenu)
                .load_collection::<UiAssets>()
                .load_collection::<BackgroundAssets>(),
        )
        .add_loading_state(
            LoadingState::new(AppState::GameLoading).continue_to_state(AppState::Game),
        );
    }
}
