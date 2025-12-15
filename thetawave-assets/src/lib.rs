//! Asset management for Thetawave.
//!
//! Uses bevy_asset_loader for managed asset loading with loading states.
//! When `progress_tracking` feature is enabled, adds iyes_progress for loading progress UI.

mod data;
mod plugin;
mod systems;

use std::fmt;

// Re-export asset types
pub use data::{
    AssetResolver, BackgroundAssets, ExtendedBackgroundAssets, ExtendedGameAssets,
    ExtendedMusicAssets, ExtendedUiAssets, GameAssets, MusicAssets, ParticleMaterials, UiAssets,
};

pub use bevy_asset_loader::mapped::AssetFileStem;

// Only export LoadingProgressEvent when progress_tracking is enabled
#[cfg(feature = "progress_tracking")]
pub use data::LoadingProgressEvent;

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
