mod data;
mod plugin;
mod systems;

pub use data::{
    AssetResolver, BackgroundAssets, ExtendedGameAssets, ExtendedUiAssets, GameAssets,
    LoadingProgressEvent, MusicAssets, ParticleMaterials, UiAssets,
};
pub use plugin::ThetawaveAssetsPlugin;
