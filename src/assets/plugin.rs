use super::data::{BackgroundAssets, MainMenuAssets};
use crate::states::AppState;
use bevy::app::Plugin;
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};

/// Plugin for managing asset loading states in Thetawave
pub(crate) struct ThetawaveAssetsPlugin;

impl Plugin for ThetawaveAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_loading_state(
            LoadingState::new(AppState::MainMenuLoading)
                .continue_to_state(AppState::MainMenu)
                .load_collection::<MainMenuAssets>()
                .load_collection::<BackgroundAssets>(),
        );
    }
}
