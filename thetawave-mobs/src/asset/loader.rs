//! Asset loader for .mob files.

use bevy::asset::{Asset, io::Reader, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use toml::Value;

use super::error::TomlAssetLoaderError;

/// Raw mob asset - holds unparsed TOML value.
///
/// This allows merging with .mobpatch files before deserializing
/// to the final MobAsset struct, avoiding unnecessary serialization.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct RawMob {
    /// Raw TOML value containing mob definition
    pub value: Value,
}

/// Asset loader for .mob files (TOML format).
#[derive(Default)]
pub struct MobAssetLoader;

impl AssetLoader for MobAssetLoader {
    type Asset = RawMob;
    type Settings = ();
    type Error = TomlAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let value: Value = toml::from_slice(&bytes)?;
        Ok(RawMob { value })
    }

    fn extensions(&self) -> &[&str] {
        &["mob"]
    }
}
