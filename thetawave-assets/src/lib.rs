mod data;
mod data_loader;
mod plugin;
mod systems;

use std::fmt;

pub use data::{
    AssetResolver, BackgroundAssets, ExtendedBackgroundAssets, ExtendedGameAssets,
    ExtendedMusicAssets, ExtendedUiAssets, GameAssets, LoadingProgressEvent, MusicAssets,
    ParticleMaterials, UiAssets,
};
pub use data_loader::load_with_extended;
pub use plugin::ThetawaveAssetsPlugin;

/// Errors that can occur during asset resolution
#[derive(Debug, Clone)]
pub enum AssetError {
    /// Asset with the given key was not found in any collection
    NotFound(String),
    /// Both extended and normal collections are empty for random selection
    EmptyCollections(String),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetError::NotFound(key) => write!(f, "Asset not found: {}", key),
            AssetError::EmptyCollections(asset_type) => {
                write!(
                    f,
                    "No assets available for random selection: {}",
                    asset_type
                )
            }
        }
    }
}

impl std::error::Error for AssetError {}
