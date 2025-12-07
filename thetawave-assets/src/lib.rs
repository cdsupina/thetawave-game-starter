//! Asset management for Thetawave.
//!
//! When `asset_loader` feature is enabled, uses bevy_asset_loader for managed loading.
//! When disabled, uses manual loading via AssetServer.

mod data;
mod plugin;
mod systems;

#[cfg(not(feature = "asset_loader"))]
pub(crate) mod manual_loader;

use std::fmt;

// Re-export asset types
pub use data::{
    AssetResolver, BackgroundAssets, ExtendedBackgroundAssets, ExtendedGameAssets,
    ExtendedMusicAssets, ExtendedUiAssets, GameAssets, MusicAssets, ParticleMaterials, UiAssets,
};

// AssetFileStem is only needed externally when asset_loader is enabled
#[cfg(feature = "asset_loader")]
pub use bevy_asset_loader::mapped::AssetFileStem;

// When asset_loader is disabled, export our String alias
#[cfg(not(feature = "asset_loader"))]
pub use data::AssetFileStem;

// Only export LoadingProgressEvent when asset_loader is enabled
#[cfg(feature = "asset_loader")]
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
