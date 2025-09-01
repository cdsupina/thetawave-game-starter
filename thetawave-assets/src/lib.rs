mod data;
mod plugin;
mod systems;

pub use data::{
    AssetResolver, BackgroundAssets, ExtendedBackgroundAssets, ExtendedGameAssets,
    ExtendedMusicAssets, ExtendedUiAssets, GameAssets, LoadingProgressEvent, MusicAssets,
    ParticleMaterials, UiAssets,
};
pub use plugin::ThetawaveAssetsPlugin;
