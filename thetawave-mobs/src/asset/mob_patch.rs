//! Raw TOML mob patch asset for field-level overrides.
//!
//! This asset type stores the raw TOML content of extended .mob files,
//! allowing partial overrides to be merged at the TOML level rather than
//! requiring complete MobAsset deserialization.

use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use bevy::reflect::TypePath;
use toml::Value;

use super::error::TomlAssetLoaderError;

/// A raw TOML patch for mob field overrides.
///
/// Unlike MobAsset, this stores the raw TOML Value, allowing partial
/// overrides without needing all required fields.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct MobPatch {
    /// The raw TOML value representing partial mob overrides
    pub value: Value,
}

/// Asset loader for .mobpatch files (raw TOML patches).
#[derive(Default)]
pub struct MobPatchLoader;

impl AssetLoader for MobPatchLoader {
    type Asset = MobPatch;
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
        Ok(MobPatch { value })
    }

    fn extensions(&self) -> &[&str] {
        &["mobpatch"]
    }
}
