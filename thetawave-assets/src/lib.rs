mod data;
mod plugin;
mod systems;

pub use data::{
    MusicAssets, AssetResolver, BackgroundAssets, ExtendedGameAssets, GameAssets,
    LoadingProgressEvent, ParticleMaterials, UiAssets,
};
pub use plugin::ThetawaveAssetsPlugin;
